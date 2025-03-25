# `prosody_config`

This crate allows one to write a [Prosody] configuration as a type-safe Rust
data structure called `ProsodyConfig`.
Calling `print(self, header: Group<LuaComment>)` on a `ProsodyConfig` returns a
`ProsodyConfigFile`, an [AST]-like `String`-based data structure representing a
[Prosody configuration file].
Calling `to_string(&self)` on a `ProsodyConfigFile` then returns the Prosody
configuration file as a `String`, which can be saved to configure Prosody.

> [!IMPORTANT]
> This crate was developped as part of the [Prose] project, with the sole
> objective of allowing the [Prose Pod API] to reconfigure a Prosody server.
> This library is therefore not feature-complete and only focuses on our own
> needs for now.
>
> We still made this code into a separate crate to facilitate a potential
> release of the library for broader use and decouple the [Prose Pod API] logic
> from [Prosody].

[AST]: https://en.wikipedia.org/wiki/Abstract_syntax_tree "Abstract syntax tree | Wikipedia"
[Prose]: https://prose.org/ "Prose homepage"
[Prose Pod API]: https://github.com/prose-im/prose-pod-api "prose-im/prose-pod-api: Prose Pod API server. REST API used for administration and management."
[Prosody]: https://prosody.im/ "Prosody IM homepage"
[Prosody configuration file]: https://prosody.im/doc/configure "Configuring Prosody – Prosody IM"
