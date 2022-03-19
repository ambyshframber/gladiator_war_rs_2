# gladiator war

gladiator war is a simple-ish discord/forum/other web platform game, designed to be played over the course of a few weeks. the basic idea is that players create fighters, and then the fighters fight each other. all interactions are determined by dice rolls, so player commitment is low. full rules are lower down in this document (i recommend reading them, so you understand what the program is doing)
this program needs to be built with cargo. read the [rust book](https://doc.rust-lang.org/book/title-page.html) to find out more

## gladiator_war_rs_2

this is the 7th gladiator war program. the first 4 were google sheets, the 5th was python, the 6th was rust. 5 and 6 were never finished
the current iteration has support for multiple simultaneous save games, automated battling and logging, and fancy terminal readouts

### usage

the program takes a list of arguments, the first of which is a command
`gladiator-war [OPTIONS] COMMAND`
valid commands are:

**help**:
prints a short help message

**list-saves**:
lists all save games in the current global data file

**new-game**:
usage: `gladiator-war [OPTIONS] new-game SEASON_NAME [PATH]`
this creates a new game with SEASON_NAME as the season name. PATH is optional, and if not provided, the file is created in the current folder with the name generated from the season name
paths can be relative or absolute, but are expanded to absolute when they are stored. this is to allow the program to be run from anywhere and still load the save files correctly

**load**
usage: `gladiator-war [OPTIONS] load INDEX COMMAND`
loads a save game and allows you to perform actions on it
this is the big one. INDEX is the index of the save game in the list (use `list-saves` to get this). COMMAND can be several things, explained in the next section

### usage of load

`load` is the main focus of the program. it allows you as the game master to actually run the game. it takes another list of arguments, the first of which is another command
valid commands are:

**info**:
prints info about the current save game. basically just the name and number of rounds played so far

**list-fighters**:
lists all fighters (alive or dead) in a nice table, complete with all stats

**add-fighter**:
usage: `... add-fighter NAME OWNER CLASS STRENGTH SPEED SKILL`
adds a fighter to the selected game. all arguments are required. see the rules documentation to learn what they mean
adding other stats here is not yet supported, so you're gonna have to edit the save file by hand. sorry.

**add-stats**:
usage: `... add-stats INDEX STRENGTH SPEED SKILL`
adds (or removes) stats to an extant fighter. INDEX is the index of the fighter, use `list-fighters` to get it
remove stats by putting a negative number here

**next-round**:
displays the next round scheduled, or informs you that there isn't one if there isn't one

**show-round**:
usage: `... show-round INDEX`
shows a given round from the past, including results

**log-round**:
usage: `... log-round INDEX`
logs a round to a file. custom log names are not yet supported, so instead the program uses the default template specified in the global data file

**arrange-match**:
usage: `... arrange-match INDEX1 INDEX2`
arranges a match between 2 fighters. INDEX1 and INDEX2 are the indexes. cannot be used while a round is scheduled

**new-round**:
generates a round. automatically creates randomised matchups while also using all predecided matchups

**run-round**:
runs the next round (if it exists). read the rules to learn what this actually entails

### options

valid options are:

**-v**: increases verbosity. can be added multiple times
**-q**: does the opposite. also works multiple times
**-l**: automatically logs rounds when they are run
**-g PATH**: PATH is the path to a valid global data file. the program will use this file instead of the default

### other considerations

the path to a default global data file can be set with the AJAL_GW_DATA_PATH environment variable. i don't know if this works on windows, though. fingers crossed

## the rules

**1: fighters**
every player has a fighter, with 3 stats: strength, speed and skill. fighters start with 12 points to spread around all 3 stats as the player sees fit. fighters can lose or gain stat points depending on the outcome of battles. fighters also have rating, which (usually) goes up if you win and down if you lose

**1.1: classes**
every fighter also has a class, which can change some things about how they behave in battle. they don't change much, and the game is still mainly up to the roll of the dice, but it just adds a bit more variety

(i will write a proper list of classes Later but for now you can get rough descriptions from `src/fighter.rs`)

**2. battles**
a battle is a (usually violent) event in which 2 fighters have a bit of a barney. a fierce scrap, if you will. a d10 is rolled for every stat and added to it. these stats are then compared to each other
for example, bob has 5 strength and rolls a 7. bill has 4 strength and rolls a 5. bob therefore has 3 higher total strength than bill, so he gains a point. had he had 5 or more greater, he would have gained 2 points. the reverse is also true
this is repeated for every stat, and at the end the fighter with the most rolls wins

**3. injuries**
if a fighter loses, they must take an injury roll. a d8 is rolled and injuries are applied according to the following table:
1: death.
2: major injury. -1 from all stats
3: arm injury. -1 strength
4: leg injury. -1 speed
5: head injury. -1 skill
6 - 8: full recovery

**4. stat increases and rating**
if a fighter wins, they get a stat increase to spend on any stat they like. i recommend not letting players increase stats past 10, but that's just a recommendation
rating does play a part in this: if the winner has rating 3 or more higher than the loser, they do not get a stat increase and also do not gain rating. the loser also does not lose rating. if they have rating 3 or more lower, they get 2, gain 2 rating, and the loser loses 2 rating. this is intended to stop one or two fighters completely dominating the competition from the early rounds

**5. arranging fights**
players may arrange fights before the match is decided. this is just intended to let some character develop in the game, and maybe instill lasting rivalries

**6. arenas and modifiers**
all battles have an arena and modifier. these make some changes, ranging from small to large

(again i don't have a list but you can check `src/round.rs`)

### happy pugiliating!