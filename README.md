A work-hours tracker

I had trouble keeping track of my work hours when moving between timezones, so I made this app :^)

Start your work session:
```
$ punch_clock in "This is me, punchin' in"
```

And when you're done with work:
```
$ punch_clock out "Okay, I'm done. I give up."
```

(The comments are optional, but very helpful for reminding yourself why you stopped working in the middle of the day three weeks ago)

I like to use an alias:
```
$ punch in
```

Your hours are stored as datetime pairs in plain text in a project root directory `.punch_clock/record`, like so:
```
2023-11-09T14:10:51.153842+00:00
2023-11-09T14:30:13.018842+00:00 No time like the present

2023-11-23T08:01:45.948514+00:00
2023-11-23T08:21:38.644514+00:00

2023-11-30T09:01:39.241707+00:00 This is me, punchin' in
2023-11-30T09:02:00.547707+00:00 Okay, I'm done. I give up.
```

Work hours can be viewed in a calendar:
```
$ punch_clock calendar 2023-06-08 2023-06-30
Total time: 80 hours, 38 minutes
2023-06-08 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓▓ 3 hours, 32 minutes
2023-06-09 ▓▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒ 1 hours, 51 minutes
2023-06-10 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░ 
2023-06-11 ░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒ 
2023-06-12 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓░▒░▓▓▓▓▓▓▓▓▒░ 4 hours, 34 minutes
2023-06-13 ░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▓▓▓▓▒░▒ 1 hours, 46 minutes
2023-06-14 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▓▓▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 8 hours, 45 minutes
2023-06-15 ░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▓▓▓▓▓▓▓▓▓ 4 hours, 35 minutes
2023-06-16 ▓░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▒░▒░▒░▒░ 1 hours, 1 minutes
2023-06-17 ░▒▓▓░▒░▒░▒▓▓▓▓░▒░▒░▓▓▓░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒ 4 hours, 52 minutes
2023-06-18 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░ 
2023-06-19 ░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▓▓▓▓▓▓▓▓▓▓▓░▒░▒ 5 hours, 46 minutes
2023-06-20 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓ 2 hours, 44 minutes
2023-06-21 ▓▓▓▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓░▒░▒░▒░▒░▒░▓▓▓▓▓░▒░▒▓▓▓▒░▒ 7 hours, 16 minutes
2023-06-22 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓░▓▓▓▓ 4 hours, 40 minutes
2023-06-23 ▓▓▓▓░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓ 5 hours, 5 minutes
2023-06-24 ▓▓▓▓▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░ 2 hours, 11 minutes
2023-06-25 ░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒ 
2023-06-26 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓░▒░▒░▒░▒░▒░ 3 hours, 8 minutes
2023-06-27 ░▒▓▓▓▒░▒░▒░▒░▒░▒░▒▓▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▒░▓▓▓▓▓ 7 hours, 21 minutes
2023-06-28 ▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▒░▒░▒░▒░▒▓▓▓▓▓ 2 hours, 56 minutes
2023-06-29 ▓▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓▓▓ 4 hours, 17 minutes
2023-06-30 ▓░▒░▒░▒░▓▓▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒░▒▓▓▓▓▓▓░▒░▒░▒░▒░ 4 hours, 8 minutes
```

Or by in a day view:
```
$ punch_clock day 2023-11-10
Total time: 0 hours, 19 minutes
00:00 ▒░▒░▒░
01:00 ░▒░▒░▒
02:00 ▒░▒░▒░
03:00 ▓▓▓▓▓▓ 03:10 -> *03:30 *"No time like the present"
04:00 ▒░▒░▒░
05:00 ░▒░▒░▒
06:00 ▒░▒░▒░
07:00 ░▒░▒░▒
08:00 ▒░▒░▒░
09:00 ░▒░▒░▒
10:00 ▒░▒░▒░
11:00 ░▒░▒░▒
12:00 ▒░▒░▒░
13:00 ░▒░▒░▒
14:00 ▒░▒░▒░
15:00 ░▒░▒░▒
16:00 ▒░▒░▒░
17:00 ░▒░▒░▒
18:00 ▒░▒░▒░
19:00 ░▒░▒░▒
20:00 ▒░▒░▒░
21:00 ░▒░▒░▒
22:00 ▒░▒░▒░
23:00 ░▒░▒░▒
```

# Install

```
cd punch_clock/
cargo install --path .
```
