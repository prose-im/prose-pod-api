# Network cheks

Symbolic links work around the fact that nested `$ref`s are evaluated by Step CI
with different working directories. Thanks to this workaround, we can `$ref`
parts of files from the `network-checks/` directory that ultimately `$ref`
other files without Step CI complaining that it doesn't find it.

TODO: Fix how `$ref`s work in Step CI in order to get rid of this.
