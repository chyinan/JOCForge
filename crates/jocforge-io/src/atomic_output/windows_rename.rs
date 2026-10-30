//! Windows handle-based, no-replace staging rename.
//!
//! The selected implementation was exercised in the host's local NTFS
//! workspace test directory. UNC, ReFS, reparse/ancestor stability, and crash
//! durability are not established by that observation.

#![allow(unsafe_code)]

use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    io,
    mem::{offset_of, size_of},
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, GENERIC_READ, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FileRenameInfo, OPEN_EXISTING, SetFileInformationByHandle,
    },
};

const DELETE_ACCESS: u32 = 0x0001_0000;

#[repr(C)]
struct FileRenameFlags {
    union_storage: u32,
}

#[repr(C)]
struct FileRenameInformation {
    flags: FileRenameFlags,
    root_directory: *mut c_void,
    file_name_length: u32,
    file_name: [u16; 1],
}

pub(super) fn open_staging_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub(super) fn open_staging_file_for_rename(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).access_mode(GENERIC_READ | DELETE_ACCESS);
    options.open(path)
}

pub(super) fn open_parent_directory(path: &Path) -> io::Result<File> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows parent-directory handle requires an absolute path",
        ));
    }
    let mut path_wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if path_wide.is_empty() || path_wide.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "parent directory path is empty or contains a null character",
        ));
    }
    path_wide.push(0);

    // SAFETY: path_wide is NUL-terminated UTF-16 and remains alive through the
    // synchronous call. Null security/template pointers request default
    // behavior. The handle is checked before ownership is transferred to File,
    // which closes the valid uniquely-owned HANDLE exactly once on drop.
    let directory = unsafe {
        let handle = CreateFileW(
            path_wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        File::from_raw_handle(handle)
    };
    Ok(directory)
}

pub(super) fn rename_file_no_replace(file: &File, destination: &Path) -> io::Result<()> {
    if !destination.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows handle rename requires an absolute destination path",
        ));
    }

    let destination_wide: Vec<u16> = destination.as_os_str().encode_wide().collect();
    if destination_wide.is_empty() || destination_wide.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination path is empty or contains a null character",
        ));
    }

    let file_name_byte_length = destination_wide
        .len()
        .checked_mul(size_of::<u16>())
        .and_then(|length| u32::try_from(length).ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination path is too long for FILE_RENAME_INFO",
            )
        })?;
    let file_name_bytes = usize::try_from(file_name_byte_length).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination path is too long for FILE_RENAME_INFO",
        )
    })?;
    let file_name_offset = offset_of!(FileRenameInformation, file_name);
    let flexible_name_length = file_name_offset
        .checked_add(file_name_bytes)
        .and_then(|length| length.checked_add(size_of::<u16>()))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination path is too long for FILE_RENAME_INFO",
            )
        })?;
    let information_byte_length = size_of::<FileRenameInformation>().max(flexible_name_length);
    let information_size = u32::try_from(information_byte_length).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "rename information is too large for SetFileInformationByHandle",
        )
    })?;
    let allocation_words = information_byte_length
        .checked_add(size_of::<u64>() - 1)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination path is too long for FILE_RENAME_INFO",
            )
        })?
        / size_of::<u64>();
    let mut information_storage = vec![0_u64; allocation_words];
    let information = information_storage
        .as_mut_ptr()
        .cast::<FileRenameInformation>();

    // SAFETY: the u64 allocation provides the C structure alignment and is at
    // least max(sizeof(FILE_RENAME_INFO), offsetof(FileName)+path bytes+NUL).
    // It remains alive across the synchronous call. The byte length excludes
    // the NUL, RootDirectory is null for the full path, and the four-byte union
    // is zero so ReplaceIfExists is false.
    unsafe {
        ptr::addr_of_mut!((*information).flags).write(FileRenameFlags { union_storage: 0 });
        ptr::addr_of_mut!((*information).root_directory).write(ptr::null_mut());
        ptr::addr_of_mut!((*information).file_name_length).write(file_name_byte_length);
        ptr::copy_nonoverlapping(
            destination_wide.as_ptr(),
            ptr::addr_of_mut!((*information).file_name).cast::<u16>(),
            destination_wide.len(),
        );
        ptr::addr_of_mut!((*information).file_name)
            .cast::<u16>()
            .add(destination_wide.len())
            .write(0);

        let succeeded = SetFileInformationByHandle(
            file.as_raw_handle().cast::<c_void>(),
            FileRenameInfo,
            information.cast::<c_void>(),
            information_size,
        );
        if succeeded == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

pub(super) fn is_destination_exists(error: &io::Error) -> bool {
    let raw_os_error = error
        .raw_os_error()
        .and_then(|code| u32::try_from(code).ok());
    raw_os_error == Some(ERROR_ALREADY_EXISTS) || raw_os_error == Some(ERROR_FILE_EXISTS)
}
