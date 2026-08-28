# Upstream Query Coverage

| Dialect | Harvested | Query Candidates | Passed | Failed | Pass Rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| bigquery | 38 | 7 | 7 | 0 | 100.0% |
| postgres | 225 | 15 | 15 | 0 | 100.0% |
| databricks | 50 | 13 | 13 | 0 | 100.0% |
| hive | 45 | 8 | 8 | 0 | 100.0% |
| snowflake | 292 | 79 | 63 | 16 | 79.7% |

## bigquery

Harvested: 38. Query candidates: 7. Passed: 7. Failed: 0.

All current query candidates pass.

## postgres

Harvested: 225. Query candidates: 15. Passed: 15. Failed: 0.

All current query candidates pass.

## databricks

Harvested: 50. Query candidates: 13. Passed: 13. Failed: 0.

All current query candidates pass.

## hive

Harvested: 45. Query candidates: 8. Passed: 8. Failed: 0.

All current query candidates pass.

## snowflake

Harvested: 292. Query candidates: 79. Passed: 63. Failed: 16.

Representative failures:

```text
sqlparser_snowflake.rs:3080
SQL: SELECT n, h, POSITION(n IN h) FROM pos
[31mError:[0m unexpected token "Some(Word(Word { text: \"h\", keyword: NoKeyword, quoted: false }))" at bytes 27..28
   [38;5;246m╭[0m[38;5;246m─[0m[38;5;246m[[0m sqlparser_snowflake.rs:3080:1:28 [38;5;246m][0m
   [38;5;246m│[0m
 [38;5;246m1 │[0m [38;5;249mS[0m[38;5;249mE[0m[38;5;249mL[0m[38;5;249mE[0m[38;5;249mC[0m[38;5;249mT[0m[38;5;249m [0m[33mn[0m[38;5;249m,[0m[38;5;249m [0m[33mh[0m[38;5;249m,[0m[38;5;249m [0m[38;5;249mP[0m[38;5;249mO[0m[38;5;249mS[0m[38;5;249mI[0m[38;5;249mT[0m[38;5;249mI[0m[38;5;249mO[0m[38;5;249mN[0m[38;5;249m([0m[33mn[0m[38;5;249m [0m[38;5;249mI[0m[38;5;249mN[0m[38;5;249m [0m[31mh[0m[38;5;249m)[0m[38;5;249m [0m[38;5;249mF[0m[38;5;249mR[0m[38;5;249mO[0m[38;5;249mM[0m[38;5;249m [0m[33mp[0m[33mo[0m[33ms[0m
 [38;5;240m  │[0m        [33m┬[0m  [33m┬[0m           [33m┬[0m    [31m┬[0m       [33m─[0m[33m┬[0m[33m─[0m  
 [38;5;240m  │[0m        [33m╰[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m `n` looks like the SQL keyword `ON`
 [38;5;240m  │[0m           [33m│[0m           [33m│[0m    [31m│[0m        [33m│[0m   
 [38;5;240m  │[0m           [33m╰[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m `h` looks like the SQL keyword `ON`
 [38;5;240m  │[0m                       [33m│[0m    [31m│[0m        [33m│[0m   
 [38;5;240m  │[0m                       [33m╰[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m `n` looks like the SQL keyword `ON`
 [38;5;240m  │[0m                            [31m│[0m        [33m│[0m   
 [38;5;240m  │[0m                            [31m╰[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m unexpected token "Some(Word(Word { text: \"h\", keyword: NoKeyword, quoted: false }))" at bytes 27..28
 [38;5;240m  │[0m                            [33m│[0m        [33m│[0m   
 [38;5;240m  │[0m                            [33m╰[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m[33m─[0m `h` looks like the SQL keyword `ON`
 [38;5;240m  │[0m                                     [33m│[0m   
 [38;5;240m  │[0m                                     [33m╰[0m[33m─[0m[33m─[0m[33m─[0m `pos` looks like the SQL keyword `ON`
[38;5;246m───╯[0m

```

```text
sqlparser_snowflake.rs:1199
SQL: SELECT * FROM ((SELECT 1) AS t)
[31mError:[0m unexpected token "Some(LParen)" at bytes 15..16
   [38;5;246m╭[0m[38;5;246m─[0m[38;5;246m[[0m sqlparser_snowflake.rs:1199:1:16 [38;5;246m][0m
   [38;5;246m│[0m
 [38;5;246m1 │[0m [38;5;249mS[0m[38;5;249mE[0m[38;5;249mL[0m[38;5;249mE[0m[38;5;249mC[0m[38;5;249mT[0m[38;5;249m [0m[38;5;249m*[0m[38;5;249m [0m[38;5;249mF[0m[38;5;249mR[0m[38;5;249mO[0m[38;5;249mM[0m[38;5;249m [0m[38;5;249m([0m[31m([0m[38;5;249mS[0m[38;5;249mE[0m[38;5;249mL[0m[38;5;249mE[0m[38;5;249mC[0m[38;5;249mT[0m[38;5;249m [0m[38;5;249m1[0m[38;5;249m)[0m[38;5;249m [0m[38;5;249mA[0m[38;5;249mS[0m[38;5;249m [0m[33mt[0m[38;5;249m)[0m
 [38;5;240m  │[0m                [31m┬[0m             [33m┬[0m  
 [38;5;240m  │[0m                [31m╰[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m unexpected token "Some(LParen)" at bytes 15..16
 [38;5;240m  │[0m                              [33m│[0m  
 [38;5;240m  │[0m                              [33m╰[0m[33m─[0m[33m─[0m `t` looks like the SQL keyword `TO`
[38;5;246m───╯[0m

```

```text
sqlparser_snowflake.rs:1203
SQL: SELECT * FROM (((SELECT 1) AS t))
[31mError:[0m unexpected token "Some(LParen)" at bytes 15..16
   [38;5;246m╭[0m[38;5;246m─[0m[38;5;246m[[0m sqlparser_snowflake.rs:1203:1:16 [38;5;246m][0m
   [38;5;246m│[0m
 [38;5;246m1 │[0m [38;5;249mS[0m[38;5;249mE[0m[38;5;249mL[0m[38;5;249mE[0m[38;5;249mC[0m[38;5;249mT[0m[38;5;249m [0m[38;5;249m*[0m[38;5;249m [0m[38;5;249mF[0m[38;5;249mR[0m[38;5;249mO[0m[38;5;249mM[0m[38;5;249m [0m[38;5;249m([0m[31m([0m[38;5;249m([0m[38;5;249mS[0m[38;5;249mE[0m[38;5;249mL[0m[38;5;249mE[0m[38;5;249mC[0m[38;5;249mT[0m[38;5;249m [0m[38;5;249m1[0m[38;5;249m)[0m[38;5;249m [0m[38;5;249mA[0m[38;5;249mS[0m[38;5;249m [0m[33mt[0m[38;5;249m)[0m[38;5;249m)[0m
 [38;5;240m  │[0m                [31m┬[0m              [33m┬[0m  
 [38;5;240m  │[0m                [31m╰[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m[31m─[0m unexpected token "Some(LParen)" at bytes 15..16
 [38;5;240m  │[0m                               [33m│[0m  
 [38;5;240m  │[0m                               [33m╰[0m[33m─[0m[33m─[0m `t` looks like the SQL keyword `TO`
[38;5;246m───╯[0m

```
