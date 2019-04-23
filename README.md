# Rust DB + Index server
Mimic a fuzzy-search key-value store DB with a persistence layer that writes to a state mutation log, which indexers can tail and answer queries about.

The fuzzy-searching uses [normalized Damerau–Levenshtein distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance) using the `strsim` crate.

# TODO
- [ ] Allow index to be started separately from DB
- [ ] Paginated log fetching
- [ ] Log compaction
- [ ] Replace DB's hashtable persistence with only the log?
- [ ] Unit tests
- [ ] Testing script

```bash
# start the db server
$ ROCKET_PORT=8000 cargo run --bin db

# start the index server
$ ROCKET_PORT=8001 cargo run --bin index

# prime the db
$ curl --header "Content-Type: application/json" --request POST --data '{"key":"a", "value":"hello"}' http://localhost:8000/db
$ curl --header "Content-Type: application/json" --request POST --data '{"key":"b", "value":"world"}' http://localhost:8000/db

# Query the index
$ curl http://localhost:8001/o/0.1
{"keys":["b","a"]}

$ curl http://localhost:8001/h/0.1
{"keys":["a"]}

$ curl http://localhost:8001/w/0.1
{"keys":["b"]}

$ curl http://localhost:8001/world/1.0
{"keys":["b"]}
```
