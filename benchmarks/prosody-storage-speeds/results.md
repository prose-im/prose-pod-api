# Prosody storages benchmark results (`2025-04-17T14:32:19+00:00`)

## `internal`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  365ms |  398ms |  382ms |
| 50       |  695ms |  741ms |  715ms |
| 75       | 1228ms | 1261ms | 1244ms |
| 100      | 1887ms | 2361ms | 1992ms |

## `appendmap`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  214ms |  221ms |  217ms |
| 50       |  304ms |  310ms |  306ms |
| 75       |  394ms |  430ms |  403ms |
| 100      |  488ms |  508ms |  497ms |

## `sqlite`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  462ms |  531ms |  482ms |
| 50       |  803ms |  819ms |  811ms |
| 75       | 1151ms | 1212ms | 1177ms |
| 100      | 1480ms | 1647ms | 1540ms |

## `sqlite journal_mode=WAL`

| member # | min.   | max.   | avg.   |
| -------- | ------ | ------ | ------ |
| 25       |  234ms |  257ms |  241ms |
| 50       |  326ms |  357ms |  332ms |
| 75       |  419ms |  449ms |  429ms |
| 100      |  519ms |  550ms |  536ms |
