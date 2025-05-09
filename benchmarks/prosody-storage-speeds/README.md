# Prosody `roster` storages benchmark

Benchmark realized in the context of debugging
[Adding members is O(n) · Issue #222 · prose-im/prose-pod-api][#222].

[#222]: https://github.com/prose-im/prose-pod-api/issues/222

Protocol:

1. Add 24 members to a team (creating a new user every time)
2. Add a 25th member, then delete it, and repeat **10 times** to perform stats
3. Reach 49 members, then perform the benchmark on the 50th
4. Repeat for the 75th and 100th

Durations represent the time it takes:

1. To call the Prose Pod API (local HTTP ReST)
2. For the Prose Pod API to create a new user in its database
3. For the Prose Pod API to make HTTP ReST calls to Prosody (in a Docker Network)
   1. Create the user
   2. Update its vCard
   3. Add it to a team

Updating all the rosters when adding the member to a team is the only O(n) operation, therefore
the results directly reflect the effect of the storage used for the `roster` module.

Scenarios:

- `internal` means we use `internal` as the default storage
- `appendmap` means we use `appendmap` as the storage for `roster`
- `sqlite-luadbi` means we use SQLite as the storage for `roster`, and make DB calls using
  LuaDBI
- `sqlite-luasqlite` means we use SQLite as the storage for `roster`, and make DB calls using
  LuaSQLite3
- `sqlite-as-default-luasqlite` means we use SQLite as the default , and make DB calls using
  LuaSQLite3
- “Delayed roster updates” means the API leveraged `mod_groups_internal:add_member`’s
  `delay_update` parameter to not synchronize the team members’ rosters and used a debounce
  to synchronize it after some time (≈10s).
