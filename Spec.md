# Syndicate JSON Canvas

Goal: to allow a user to take a [JSON canvas file](https://jsoncanvas.org/spec/1.0/) (for example, created via Obsidian) and syndicate the contents to Twitter, a microblog or another sink. Each node in the JSON canvas can be mapped to a blog

There are a few features:
1. It should be designed as libraries and binary that can be run do the syndication end to end. The libraries should contain the important logic, while the binary source should merely string them together; effectively the binary source is just a configuration that defines the sources / sinks for the content.
2. The library should enable watching the file and running the syndication steps as needed.
3. The library should decompose into two parts: two part that parses, filters, and enriches the nodes in the jsoncanvas file (call this the backend), and the part that actually makes the edits to the syndication syncs (call this the frontend)
4. To begin with, we will not handle updating or deleting, only creating/publishing to syndication sinks.
5. However, we must ensure we don't duplicately publish something. To ensure this, we should maintain a set of jsoncanvas `NodeId`s in memory (and synced to disk for persistence). This should be read.

## Backend functionality

### Current

1. Critically, the backend should allow filtering the nodes. Currently the default filter just checks that the node is a `TextNode`, has a non-empty `text` field, and is colored red before publishing it
2. Besides that, the backend should provide another function to marshal the type to

### Changes Desired

1. In reality, i want you to convert these two functions into one which we can use inside a `filter_map`.
2. Also, things don't build

## Code structuring

1. Each piece of content to be syndicated should be coerced to the `SyndicationFormat` type. This is a type shared between the backend the frontend.

2. **IMPORTANT: Simplify lifetimes in SyndicationFormat**. Instead of storing references with lifetimes, store NodeIds and EdgeIds. These are cheap to clone and can be converted to CoW strings when needed. This avoids complex lifetime issues and makes the code much simpler.

3. A syndication sink should be a trait:

```rust
trait SyndicationSink {
  fn publish(&mut self, item: &SyndicationFormat, dry_run: bool) -> Result<(), SyndicationError>;
  fn name(&self) -> &str;
}
```

The trait should support a **dry-run mode** where `dry_run: bool` determines whether to actually perform the syndication or just log what would happen. This is useful for testing and validation.

4. We need a new library crate for the frontend. This crate should define the relevant trait, but also provide an implementation for Twitter and for git commiting + pushing files to a repository (by taking a path to the repository on disk)

## Libraries to use

1. For now, remove the dependence on clap. The expectation should be that the user hard codes the path to the jsoncanvas (`.canvas`) file.
2. Keep using notify, but add debouncing: https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/ - read this file.
3. Keep using the jsoncanvas crate
4. **Use `thiserror` for error handling in the library crate**. Define a proper `SyndicationError` type instead of using `Box<dyn Error>`. 

## Committing using Jujutsu (a git compatible VCS)

The is a jujutsu (`jj`) repository: https://docs.jj-vcs.dev/latest/cli-reference/

Break your changes into bitesized pieces (they should mostly do one thing) as jj revisions. Run `jj show` to see your changes in the current revision. Run `jj describe -m <your commit message>` once your done with a revision and `jj new` to create a new one on top

Occassionally I will ask you to try a different approach. In this case, I will typically run `jj new` from an older commit, so that you will not have the previous approach on hand.

## DO THIS STUFF

1. First, I want you to fix the errors (there are a bunch of them).
2. Explore the project.
3. Next (or maybe concurrently with the first thing), think about what improvements can be made to the current design. Tell me about them!
4. Implement the other stuff
