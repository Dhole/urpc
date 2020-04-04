# uRPC

> uRPC (pronounced micro RPC) is a simple and lightweight RPC framework designed
> with embedded systems in mind.  The server side works in a heapless
> environment with `no_std` and is supposed to use very low resources.  The
> current client side implementation requires `std`.

[![crates.io](https://img.shields.io/crates/v/urpc.svg)](https://crates.io/crates/urpc)
[![crates.io](https://img.shields.io/crates/d/urpc.svg)](https://crates.io/crates/urpc)
[![Released API docs](https://docs.rs/urpc/badge.svg)](https://docs.rs/urpc)

## Features

- [x] Support for 256 different methods.
- [x] Low level primitives for the server side regarding request parsing and
  reply serializing.
- [x] Single argument of any `sedre::Serialize + serde::DeserializeOwned` type.
- [x] Optional byte buffer for the argument that doesn't involve any buffer copy.
    - This feature is designed to optimize the transfer of bytes between client
      and server minimizing the amount of used memory in the server.
- [x] Single return value of any `sedre::Serialize + serde::DeserializeOwned` type.
- [x] Optional byte buffer for the reply ~~that doesn't involve any buffer copy~~.
    - This feature is designed to optimize the transfer of bytes between client
      and server ~~minimizing the amount of used memory in the client~~.
- [ ] Methods can return custom errors.
- [ ] Asyncrhonous methods.
    - [ ] Support for holding 255 async uncompleted requests.
- [ ] Stream methods.

## Notes

The current client implementation requires the `paste` crate.  Add this to your `Cargo.toml`:
```
[dependencies]
paste = "0.1.7"
```

## Packet format

- The request packet consists of a 7 byte header, an optional body and an
  optional byte buffer.
- The reply packet consists of a 6 byte header, an optional body and an
  optional byte buffer.

### Request Header

length | desc
-------|-----
8b | method index
8b | channel id
8b | options
16b | body length (little endian)
16b | optional buffer length (little endian)

### Reply Header

length | desc
-------|-----
8b | channel id
8b | options
16b | body length (little endian)
16b | optional buffer length (little endian)

## Examples

See [examples/requests.rs](requests.rs)

## License

The code is released under the 3-clause BSD License.
