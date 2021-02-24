# Admin Web Panel

This is the management portal for the CDL, useful for updating
configuration, manipulating data.


## Setup

This site is written with [Svelte.JS][svelte] and [TypeScript][ts].
To run or develop this site, you'll need to install [NPM][npm]:
I recommend using a version manager to install the latest version
like [fnm][fnm], a Rust-based version manager for NPM.

Once you have NPM in your path, run `npm i` in this directory to
install all package dependencies.


## Running

For development, the command `npm run dev` will run a dev server
on localhost:5000 (or a random port if 5000 is taken) which you
can access from your local browser.


## Deployment

This site is deployed as a plain folder, specifically the `public`
folder in the root of this repo. Before deploying that folder, make
sure to run `npm run build` to build an optimized version of this
site and save it to the `public` directory.


[svelte]: https://svelte.dev/
[ts]: https://www.typescriptlang.org/
[npm]: https://npmjs.com/
[fnm]: https://github.com/Schniz/fnm
