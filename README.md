# Onlivfe core

Onlivfe core is the a library that provides the models & some of the logic for onlivfe.
It's usage is not supported outside of the onlivfe ecosystem, though we won't stop others from using it.
Also note that the license is [AGPL](https://tldrlegal.com/license/gnu-affero-general-public-license-v3-(agpl-3.0)).

## Development

Basic requirements:

- [Git](https://git-scm.com)
- [Rust](https://www.rust-lang.org/)

### Building

Start off by cloning the project with git.

```sh
git clone https://github.com/onlivfe/core
```

Then open the project folder in your terminal, & run `cargo build`.
Then get to hacking, & optionally replace the dependency in other projects by [overriding dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html).

### Storage

The `core` abstracts away storage backends, so that consumers can optionally provide their own.
This is done so that for example in the future `core` could easily be used via WebAssembly,
as long as there is a compatible storage backend.

In the short term, there's plans for a in-memory caching backend, which should be enough to start developing other parts of the system.
In the long term, a standard database backend is being developed, with proper support for historical data.

### API considerations

Note that [`serde`](https://serde.rs/) by default fails at deserializing the whole response if even a single part is unexpected.
Which is why we should generally allow data to just be missing, and use [`serde_with`](https://serde.rs/)'s [`VecSkipError`](https://docs.rs/serde_with/latest/serde_with/struct.VecSkipError.html) and [`DefaultOnError`](https://docs.rs/serde_with/latest/serde_with/struct.DefaultOnError.html).

Beyond that, we should try to respect the platforms that we interact with, and generally follow rate limits and/or wishes of said platforms developers, up to a reasonable extent.
However, as our application is meant for interoperability, which some platforms may not like, thus we must not bow down to any singular platform too much either.
