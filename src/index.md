# Rust Forge
Welcome to the Rust Forge! Rust Forge serves as a repository of supplementary
documentation useful for members of [The Rust Programming Language]. If
you find any mistakes, typos, or want to add to the Rust Forge, feel free to
file an issue or PR [on GitHub].

[The Rust Programming Language]: https://rust-lang.org
[on GitHub]: https://github.com/rust-lang/rust-forge

### Help Wanted

Want to contribute to Rust, but don't know where to start? Check out [this guide](./how-to-start-contributing.md).

### Current Release Versions

<!-- All `<span id="..."></span>` elements are filled at run time when a reader
visits the website. Please refer to `js/index.js` for how these values
are generated.

Avoid changing the "Current Release Versions" without also updating the selector
in `js/index.js`.
-->

Channel    | Version | Will be stable on | Will branch from master on |
-----------|---------|-------------------|----------------------------|
Stable     | <span id="stable-version"></span>  | <span id="stable-release-date"></span>  | <span id="stable-branch-date"></span>
Beta       | <span id="beta-version"></span>    | <span id="beta-release-date"></span>    | <span id="beta-branch-date"></span>
Nightly    | <span id="nightly-version"></span> | <span id="nightly-release-date"></span> | <span id="nightly-branch-date"></span>
Nightly +1 | <span id="next-version"></span>    | <span id="next-release-date"></span>    | <span id="next-branch-date"></span>

See the [release process](./release/process.md) documentation for details on
what happens in the days leading up to a release.

### No Tools Breakage Week
To ensure the beta release includes all the tools, no [tool breakages] are
allowed in the week before the beta cutoff (except for nightly-only tools).

Beta Cut | No Breakage Week
---------|-----------------
<span id="nightly-cycle"></span> | <span id="nightly-timespan"></span>
<span id="next-cycle"></span>    | <span id="next-timespan"></span>

[tool breakages]: ./infra/toolstate.md

### External Links

* [Bibliography] of research papers and other projects that influenced Rust.
* [Rust Pontoon] is a translation management system used to localize the Rust
  website.

[Bibliography]: https://rustc-dev-guide.rust-lang.org/appendix/bibliography.html
[Rust Pontoon]: https://pontoon.rust-lang.org/
