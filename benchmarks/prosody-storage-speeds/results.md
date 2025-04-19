# Prosody storages benchmark results (`2025-04-18T16:06:20+00:00`)

## `internal`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  377ms |  410ms |  387ms |
| 50       |  744ms |  820ms |  771ms |
| 75       | 1251ms | 1528ms | 1318ms |
| 100      | 1891ms | 2014ms | 1950ms |

## `appendmap`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  217ms |  222ms |  219ms |
| 50       |  304ms |  314ms |  309ms |
| 75       |  397ms |  421ms |  405ms |
| 100      |  496ms |  543ms |  515ms |

## `sqlite-luadbi`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  467ms |  483ms |  473ms |
| 50       |  804ms |  816ms |  811ms |
| 75       | 1078ms | 1181ms | 1153ms |
| 100      | 1506ms | 1596ms | 1546ms |

## `sqlite-luadbi journal_mode=WAL`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  230ms |  238ms |  235ms |
| 50       |  318ms |  347ms |  328ms |
| 75       |  418ms |  442ms |  422ms |
| 100      |  518ms |  544ms |  526ms |

## `sqlite-luadbi sqlite_tune="normal" (journal_mode=WAL)`

`sqlite_tune="normal"` didn’t work (`journal_mode` is `delete`).

## `sqlite-luasqlite sqlite_tune="normal" (journal_mode=WAL)`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  266ms |  281ms |  274ms |
| 50       |  359ms |  397ms |  370ms |
| 75       |  486ms |  511ms |  494ms |
| 100      |  598ms |  619ms |  606ms |

## `sqlite-as-default-luasqlite sqlite_tune="normal" (journal_mode=WAL)`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  194ms |  203ms |  198ms |
| 50       |  249ms |  259ms |  253ms |
| 75       |  378ms |  391ms |  382ms |
| 100      |  396ms |  480ms |  465ms |
