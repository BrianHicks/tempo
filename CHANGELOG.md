# 1.1.0

- "pull" is renamed "ready" (but `tempo pull` will still work, at least for now!)
- added ID to "ready" output
- added "all" command to get all the items in the store
- parsed dates are now interpeted using the local time zone instead of UTC
- allow "today" as a value in `--next` arguments
- allow cadence values in `--next` arguments (e.g. you can now say `tempo add X --next 1w` to schedule the first iteration a week out)
- improve output of "add" to be less confusing
- allow filtering by tag in "all"

# 1.0.0

Initial release!
Tempo can add, edit, pull, finish, and delete items.
