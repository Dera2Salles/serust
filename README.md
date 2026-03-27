# ftp_server

A simple, fast, and lightweight asynchronous file server written in Rust using Tokio.

`ftp_server` is a custom TCP-based file server that allows multiple users to upload, download, and list files over the network.

---

## Features

- **Asynchronous** architecture with Tokio (no threads)
- Multi-client support (concurrent connections)
- Simple custom protocol
- Basic authentication (planned)
- Upload and download files
- Clean modular architecture

---

## Protocol Commands

| Command                        | Description                          | Example                          |
|-------------------------------|--------------------------------------|----------------------------------|
| `LOGIN <username> <password>` | Login to the server                  | `LOGIN alice secret123`         |
| `LIST`                        | List files in current user's storage | `LIST`                          |
| `UPLOAD <filename> <size>`    | Upload a file (followed by data)     | `UPLOAD photo.jpg 2048576`      |
| `DOWNLOAD <filename>`         | Download a file                      | `DOWNLOAD document.pdf`         |
| `QUIT`                        | Disconnect from the server           | `QUIT`                          |

---

## Quick Start

### 1. Clone & Build

```bash
git clone https://github.com/Dera2Salles/ftp_server 
cd ftp_server
cargo build --release