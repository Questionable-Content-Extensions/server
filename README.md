# qcext-server

## Usage

```bash
npm install
cargo run
npm start
```

## Deploy to Heroku

### Manual

Using custom buildpack [heroku-buildpack-rust](https://github.com/alexschrod/heroku-buildpack-rust)

```bash
heroku buildpacks:set https://github.com/alexschrod/heroku-buildpack-rust
heroku buildpacks:add --index 1 heroku/nodejs
```
