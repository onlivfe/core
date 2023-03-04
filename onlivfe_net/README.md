# `onlivfe_net`

The network interaction parts of onlivfe.

## API considerations

[`serde`](https://serde.rs/) by default fails at deserializing the whole response if even a single part is unexpected.
Which is why we should generally allow data to just be missing, and use [`serde_with`](https://serde.rs/)'s [`VecSkipError`](https://docs.rs/serde_with/latest/serde_with/struct.VecSkipError.html) and [`DefaultOnError`](https://docs.rs/serde_with/latest/serde_with/struct.DefaultOnError.html).

Beyond that, we should try to respect the platforms that we interact with, and generally follow rate limits and/or wishes of said platforms developers, up to a reasonable extent.
However, as our application is meant for interoperability, which some platforms may not like, thus we must not bow down to any singular platform too much either.
