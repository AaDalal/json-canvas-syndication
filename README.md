# Turn thoughts in JSON Canvas (Obsidian Canvas) into Tweets or microblogs posts

## Why

Because a JSON canvas (easily made using obsidian) is a great way to organize thoughts and the connections between them; and sometimes those thoughts are best shared with others.

## How it works

This reads a JSON canvas file and syndicates them to wherever you want (e.g., the currently supported options are Twitter or a jujutsu repository). The places you publish to are called *syndication sinks*.

The cool part is that you can pick which things in the graph you want to publish. E.g., I do this by only publishing nodes that are colored red. I also have it configured so that your posts include links to the things that you connect to them.

*Note*: I haven't tested the twitter one yet, so that may need some work!

## How to use it

### Software you'll need

You'll need the `cargo` command-line tool which is how you run things in the rust programming language. You can install it here: <https://rust-lang.org/learn/get-started/>

Optionally, you can install `just`, which makes it possible to run the commands from the `./justfile`. These are just shortcuts. For instance, `just run` is a shortcut for `cargo run --release` so you don't have to remember that every time. 

For the rest of this we'll use the `just` commands, but you can look in the `./justfile` and copy the commands corresponding to what we want to run.

### How to configure it

The things you can configure are:
1. The path of the json-canvas file (e.g., `/Users/<your username>/Documents/Thoughts.canvas` - they usually end in .canvas)
2. The syndication sinks to publish to & their configurations

It's configurable by editing the code in `./src/main.rs` (I tried to make it so the main.rs file can be really simple & ideally pattern-matchable by someone who doesn't know a lot about the rust programming language).

### Keep it running in the background (even when you restart your computer)

First, you can run it in your terminal by navigating to this folder where this repository lives and running `just run` (or `cargo run --release` if you don't have `just`). While this is running, it will watch the canvas and publish to the syndication sinks when it sees changes.

**But** the previous has the downside of not working when you shut down your computer. To make it keep running between restarts, we need to add a launch agent (using a system called launchd which makes sure the program restarts when we restart the computer)

1. Copy the example plist file to `./com.syndicate-json-canvas.plist` and edit it to point to this repository
2. Run `just install` to build, copy the plist to `~/Library/LaunchAgents`, and load the service. You might get a prompt asking if the new background service should have access to wherever your json canvas is.

If you want to make sure it's all working then run `just status`. You should see something like this:

```
44601   -9      com.syndicate-json-canvas
```

(the numbers might be a bit different, but the important part is that you get a line back & the command doesn't just output nothing)
