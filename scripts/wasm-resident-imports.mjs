// pattern: Imperative Shell
// Resident encodes push their bytes through the ABI: there is no host file to read.
export const residentImports = {
  env: {
    jocforge_host_len: () => 0n,
    jocforge_host_read: () => -1,
  },
};
