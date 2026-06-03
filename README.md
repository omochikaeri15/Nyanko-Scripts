# BCC-Scripts
**DISCLAIMER:** This repository and it's binaries are for convenience and convenience only. I will **not** maintain this repository or any of the binaries under it to any reasonable degree. Use at your own risk.

BCC-Scripts is a collection of standalone pure-rust command line binaries made to be lightweight and perform specific functions. All of these tools are detached from and **DO NOT** utilize both Java and Python, providing consistent user experience as well as output.

Commands using these binaries may have to be prefixed with `./` or `.\` depending on your operating system.

## BCC-Pack

**COMMAND:** `bcc-pack`

Decrypts `.pack` / `.list` files. Can target `PACK` and `LIST` directly, can walk through a given `DIR`, or can sift through `ZIP`/`APK`/`XAPK`/`APKM`/`APKS`.

Validates regional keys and iv against known MD5 Hashes, while also validating file output and skipping corrupted junk.

### Disclaimer

>These are unofficial, educational tools. For full details regarding PONOS Corp. copyrights, authorized handling of game files and decryption keys, and our strict liability disclaimer, please read LEGAL.md before use.
