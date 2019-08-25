# naps/notebook

## Development

Four tabs:

```
RUST_BACKTRACE=1 cargo watch --exec "run" -i "frontend/*"
```

Now you can open http://localhost:8080/

```
cd frontend
yarn start # development
```

Now you can open http://localhost:3000/

```
cd frontend
# Whenever schema is rebuilt. TODO move this to build.rs ?
curl http://localhost:8080/schema.json > schema.json && node output-schema.js
```

```
cd frontend
yarn relay --watch
```

## Links

* https://github.com/splitbrain/ReMarkableAPI/wiki/Storage

## License

MIT
