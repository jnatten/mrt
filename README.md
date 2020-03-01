# mrt - Multi Repo Tool
[![Build Status](https://travis-ci.org/jnatten/mrt.svg?branch=master)](https://travis-ci.org/jnatten/mrt)

A tool to interact with multiple repositories, by executing the specified commands in each context with the specified tag (See EXAMPLES in [usage section](#usage)).

This is not a finished product and there is a chance you will encounter bugs or problems. 

Reports of [issues](https://github.com/jnatten/mrt/issues/new) are really appreciated, along with pull-requests and suggestions for new features and how to improve the code (Rust is still very new to me).

### Installation

Right now there aren't any releases, but you can compile the code yourself with these few steps:

1. Install rust (I like [rustup](https://www.rust-lang.org/tools/install) for this)
2. Clone this repository:

    `$ git clone https://github.com/jnatten/mrt.git`
3. Compile the source code with cargo from within the repository:

    `$ cd mrt && cargo build --release`
4. Move `mrt` executable to somewhere on PATH, On linux this can be something like /usr/bin/mrt

    `$ mv target/release/mrt /usr/bin/mrt`
    
5. Done! See [usage](#usage):

### Usage
```
Multi Repo Tool 0.0.1

USAGE:
    mrt [FLAGS] [+tag ..] [--] [command]

FLAGS:
    -c, --continuous-output    Will make output from commands executed in parallel with --parallel argument print to
                               terminal before every command has been executed.
    -h, --help                 Prints help information
    -l, --list-tags            List all specified +tag's and paths that are tagged...
    -P, --panic-on-nonzero     Makes mrt quit if it encounters a non-zero exit code.
    -p, --parallel             Execute at each tagged path in parallel
                               This stores output until all executions are finished and then prints them in sequence,
                               unless --continuous-output specified.
    -s, --shell                Will make command be executed in the context of a shell.
                               IE: `bash -c '<command>'`
                               `powershell /C '<command>' on windows.
    -V, --version              Prints version information

SUBCOMMANDS:
    config    Subcommand to add and remove tags, generally configure mrt itself
    help      Prints this message or the help of the given subcommand(s)
    status    Status of directories with specified tags

EXAMPLES:
    # Tag current directory with tag `backend`
    $ mrt config -a backend

    # Remove tag `backend` from current directory
    $ mrt config -d backend

    # List tagged directories
    $ mrt -l

    # Execute command in all directories tagged with `backend`
    $ mrt +backend sed -i 's/someversion = "1.0.0"/someversion = "1.2.0"/g build.sbt

    # Execute command in all directories tagged with `backend` in parallel
    $ mrt -p +backend git pull

    # Execute command in all directories tagged with `backend` and `frontend` in parallel
    $ mrt -p +backend +frontend git pull

    # List status of all directories tagged with `backend`
    $ mrt +backend status

    # Removes the `backend` tag entirely, leaving the directories intact
    $ mrt config -D backend

    # Removes all tags from current directory
    $ mrt config -r

    # Execute command in specified directory
    $ mrt +/opt/somedir ls -l

```

### Configuration
Configuring tags are mostly done with the `mrt config` command.
See examples at `mrt -h` or `mrt config -h` for more help.

##### Config file
The config file is by default located at `<HOME>/.mrtconfig.json` and is a json file.
The fastest way to add multiple directories under multiple tags and such is probably editing this file by hand.
The format is like this:
```
{
  "version": "0.0.1",
  "tags": {
    "tag1": {
      "paths": [
        "/home/user/dir1",
        "/home/user/dir2",
        ...
      ]
    },
    "tag2": {
      "paths": [
        "/home/user/dir2",
        "/home/user/dir3",
        ...
      ]
    },
    ...
  }
}
```

##### Environment variables
Some environment variables can be used to modify `mrt`'s behavior. Here's a list of them:

- `MRT_DEFAULT_TAGS` - A comma separated list of tags that should be used when no tags are specified on the command line.
    - Example: `MRT_DEFAULT_TAGS=backend,frontend`
- `MRT_CONFIG_PATH` - Where the mrt config path is located.

### Why?

I work on many repositories with similar code in some sort of a microservice environment. 
I felt the need to reduce the overhead of maintaining several repositories with similarities.

Also I think rust is an interesting language which I want to learn more about.

### Development
##### Useful commands
- **Compile**: `cargo build` 
- **Compile + Run**: `cargo run -- -h` 
- **Run tests**: `cargo test`
##### Subcommands
Subcommands _should_ be relatively easy to implement. See the subcommands [README](src/subcommands/README.md) for more info.

### Alternatives

This is not a new idea and similar projects exist.
This list is probably not complete, but here are a few alternatives and a short summary of why I still felt compelled to make this.

- https://github.com/mixu/gr - Not actively maintained, no parallelization.
- https://github.com/tobru/myrepos - Not actively maintained, seems to be only for version control.
