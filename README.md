# FTP Server Project

### NOTE: *The project currently supports only gFTP FTP client and has only a few FTP commands and is a work in progress. The TLS and SSL support is not implemented yet.*

## Overview

This project is an FTP server implemented in Rust. It supports various FTP commands and provides functionalities for file transfer, directory management, and user authentication. The server is designed to handle multiple clients concurrently using asynchronous programming with `tokio`.

## Features

- **File Transfer**: Upload and download files.
- **Directory Management**: Create, delete, and list directories.
- **User Authentication**: Basic user authentication.
- **Asynchronous Handling**: Efficient handling of multiple clients using `tokio`.

## Dependencies

The project uses the following dependencies:

- `tokio`: Asynchronous runtime for Rust.
- `serde`: Serialization and deserialization framework.
- `dotenv`: Load environment variables from a `.env` file.

## FTP Commands

### USER

**Description**: Specify the user for authentication.

**Usage**: `USER <username>`

**Example**:

USER anonymous


### PASS

**Description**: Specify the password for authentication.

**Usage**: `PASS <password>`

**Example**:

PASS guest

### QUIT

**Description**: Terminate the connection.

**Usage**: `QUIT`

**Example**:

QUIT


### RETR

**Description**: Retrieve a file from the server.

**Usage**: `RETR <filename>`

**Example**:

RETR example.txt

### STOR

**Description**: Store a file on the server.

**Usage**: `STOR <filename>`

**Example**:

STOR example.txt

### DELE

**Description**: Delete a file from the server.

**Usage**: `DELE <filename>`

**Example**:

DELE example.txt

### RMD

**Description**: Remove a directory.

**Usage**: `RMD <directory>`

**Example**:

RMD /example_directory

### MKD

**Description**: Create a directory.

**Usage**: `MKD <directory>`

**Example**:

MKD /example_directory

### PWD

**Description**: Print the current working directory.

**Usage**: `PWD`

**Example**:

PWD

### LIST

**Description**: List files in the current directory.

**Usage**: `LIST`

**Example**:

LIST

### SIZE

**Description**: Return the size of a file.

**Usage**: `SIZE <filename>`

**Example**:

SIZE example.txt

Ensure that you have a `.env` file with the necessary environment variables, such as `ROOT_DIR` for the server's root directory.

## Usage

To run the server, use the following command:

```bash
cargo run
```