# Subcommands
Creating a subcommand should be a relatively simple process.
The [config](config.rs) subcommand could be a good place to start if you are looking for examples.

### Structure
The structure of a subcommand can be largely how you want.
The only limitation is that it needs to have a function of sorts which returns a `MrtSubcommand` value (So the subcommand will somehow need to fit into a `MrtSubcommand` struct).

The output from the `MrtSubcommand` function should be added to the `Vec` returned by `get_subcommands`.

_Thats it!_ The implementation after that is entirely up to the subcommand author.

