# Cross-ADM DEE header repeatability

Two additional user-authorized ADM BWFs, First Love and Hortus Regis Alti, were encoded sequentially by DEE with the matched 1024 kbps / 5.1 / film_light / DI=false-100 / LoRo / auto-trim settings. Wash independently rehashed both input files and verified the R7 receipt package in normal and python -O modes.

The run receipts report successful E-AC-3 outputs. Only each first six-byte syncframe header was inspected; the output files were removed after hashing/header extraction. Both headers report 48 kHz, six blocks, acmod 7, and LFE present. All three ADM CHNA/AXML graphs share the same ten-row DirectSpeakers bed map, with the 5.1 subset in rows 0–5 and LFE at row 3.

This supports repeated ADM metadata shape and DEE job admission across three inputs. It does not measure PCM parity or prove DEE runtime row selection or repeated LFE residual behavior.
