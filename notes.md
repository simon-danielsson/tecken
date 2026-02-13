## todo

- [ ] the WPS is currently being too harshly punished by the number of errors a user makes

- [x] fix rendering of exercise text to screen, so that it sits nicely in the center. The length of the total exercise text will need to be calculated before adding coordinates to each line, instead of doing it backwards like it is now. I'm guessing that will be the quickest fix
- [x] fix the "-w" flag. it does work, but only when the user wants to add 100 or more words; doing "-w 6" for example doesn't work.
- [x] add endless mode functionality
- [x] add help subcommand
- [x] instead of up in the corner, all the ui should sit in the center of the viewport
- [x] make the ui a bit fancier with a frame and such
- [x] add linebreaks so that multi-line exercises are supported (this is essential if the user wants exercises with 100+ words)
- [x] take arguments at launch

## brainstorming

**== Regarding linebreaks ==**  
  
to make linebreaks a possibility I guess I'd need to refactor the exercise to be a
vector of <line, Pos(col, row)> and then have the user buffer automatically follow those possitions?

**== Arguments ==**  
  
``` bash
-w <int> : word quantity (default: 12)
-e : endless mode (exit with esc or ctrl-c)
-l <language>: language [english | swedish] (default: english)
-d <int>: difficulty [0 (200 most common) | 1 (1000 most common) | 2 (5000 most common) | 3 (10000 most common) | 4 (25000 most common)] (default: 1)
```

default args:
-w 10 -l english -d 1

**== Results ==**  
  
program will exit automatically when the exercise is finished and print a result

WPM: (amount of words in exercise - amount of errors) / minutes
Accuracy (%): 1 - (amount of errors / total characters of exercise)
Time (sec)
