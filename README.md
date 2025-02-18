# Group Activity Bot

Group Activity Bot (GAB) is a Telgram bot that will keep track of user activities in groups, e.g.

<img width="1155" alt="Screenshot" src="https://user-images.githubusercontent.com/16304728/153413001-c55f3f46-e0f1-4661-9591-a9e1ed505892.png">

**GAB only works in group and must be a group admin in order to receive each user message in a group**

## Features
|Command|Action|
|:-|:-|
|`/groupstats`|Returns a message containing the total number of messages exchanged in the group and the percentege of messages per user, with nice emojis for the top 3 users|
|`/userstats <username>`|Returns a message containing the percentage of messages of a specific user in a group|
|`/statsfile`|Returns a .csv file containing `(message timestamp, username)` for all the messages exchanged in a group, this is helpful to plot graphs and other analytics with other softwares|

### Run
```
docker compose up -d
```