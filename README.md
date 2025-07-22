# walkpwd

**walkpwd** is a minimal and fast terminal-based password manager written in Rust.

## Features

- Stores passwords locally in a JSON file
- Secure random password generation
- Automatically copies password to clipboard (Wayland, X11, macOS supported)
- Simple CLI for listing, adding, retrieving, and deleting passwords
- No external servers or internet required

## Installation

### Requirements

- Rust toolchain (`rustup`)
- Recommended system binaries:
  - `wl-clipboard` (for Wayland)
  - `xclip` or `xsel` (for X11)
  - `pbcopy` (macOS)

### Local install

Clone the repo and install with:

```bash
cargo install --path .
```
## Planed features
- AES-256 encryption of the password vault

- Master password authentication (with optional Argon2 or PBKDF2 key derivation)

- Vault file integrity chec

- Export and import options with encryption


## Usage

### Init
`walkpwd init` - initialize the vault

### Add
`walkpwd add --name <Name>` - Add a new password entry

###### Add params:
```
    (Required) Set the name of the entry
        --name,-n
    (Optional) Set a custom password
        --password,-p
    (Optional) Set a length for the password
        --length,-l
    (Optional) Use symbols to generate the password
        --use-symbols,-u
```

### Get
`walkpwd get --name <Name>` - Get the given entry password

###### Get params:
```
    (Required) Set the entry name
        --name,-n
    (Optional) Reveal the password in terminal
        --reveal,-r
```
### List
`walkpwd list` - List all the entries stored in the vault

### Delete
`walkpwd delete --name <Name>` - Delete the given entry from the vault

###### Delete params:
```
    (Required) Set the entry name to delete
        --name,-n
```

