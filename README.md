# Tempo

Tempo helps you do the things that matter to you by helping you discover the right schedule and reminding you at just the right times.

The goal, basically, is to help you remember to do what matters to you with a minimum of effort on your part.
Using Tempo should feel joyful and effortless.

How about some examples?

- **Have a favorite journaling prompt?**
  Add it to the system and Tempo will find the right time for you to come back to it.
  Or, if it turns out you don't like the prompt, it'll make sure you don't see it for a long time.

- **Let's develop some ideas!**
  It's helpful to look at your WIP writing every so often.
  Tempo will schedule quicker repetitions for productive writing sessions while deferring ideas that still need time to marinate.

- **How do you remember bookmarks and references?**
  If you're anything like me, you probably have a big bucket of bookmarks you never look at or revisit, even when they go stale or become irrelevant.
  Instead, let Tempo grab a handful to review every day.
  If they're useful, they can show up more often.
  If they're not, they'll get scheduled far into the future.

- **How about balancing side projects?**
  Just like the things above, active projects will show up sooner and inactive projects will be deferred but not forgotten.

- **Stay in touch with other people** with journaling ("Think about a nice time you had with Marcus") or contact prompts ("Have you talked with Julie recently? Maybe it'd be good to text her.")

- **Want to stay up on interesting topics?**
  Prompt yourself: "find developments about […]", or just "go check this site for interesting things about […]".

## Should I use this?

You can if you want (and can follow the license restrictions), but note that Tempo is both a work-in-progress and a [home-cooked meal](https://www.robinsloan.com/notes/home-cooked-app/).
As such, I'm open to contributions or modifications, but I'm unlikely to make a lot of huge changes that fit some use case that I don't personally have.
(However, if you try to use Tempo to do the things described above and can't, I wanna hear about it!)

## How do I use it?

To start with, add an item:

```bash
$ tempo add "What are my strengths? How can I use them?" --tag journaling
Added "What are my strengths? How can I use them?" with ID 1. Currently scheduled 1d from now, on Thursday, January 13, 2022
```

Then, every day, pull the things that are due:

```bash
$ tempo pull
What are my strengths? How can I use them? (due Thursday, January 13, 2022)
```

Then you do the prompt—whatever it means for that particular prompt—and tell the system you've completed it.
You give feedback on the cadence at the same time by passing `later`, `earlier`, etc (see the `--help` output.)

```bash
$ tempo finish 1 just-right
Finished! For next time, I bumped the schedule by <1h so the next time you'll see this will be Thursday, January 13, 2022
```

You can also say that it was too early or too late to complete the task (again, whatever that means to you!)

```bash
$ tempo edit 1 --bump later
Bumped schedule by ~2d to Saturday, January 15, 2022
```

Or drop it entirely:

```bash
$ tempo deleted 1
Deleted item with ID 1
```

### JSON Output

If you want to build other tools on top of this, add the `--format json` flag at the beginning of any command.
For example:

```bash
$ tempo --format json add "What are my strengths? How can I use them?" --tag journaling | jq .
{
  "id": 2,
  "text": "What are my strengths? How can I use them?",
  "tag_id": 1,
  "cadence": {
    "days": 1
  },
  "next": "2022-01-13T16:04:57.044733Z",
  "integral": 0,
  "last_error": 0
}
```

## How Does it Work?

Every item in Tempo has a cadence and a next due date (you can specify both of these when adding an item; guesses are fine!)
Over time, Tempo will refine the cadence based on your feedback (the `--bump` arguments above.)

Generally, you don't have to worry about the technical details: you just say if something showed up too early, too late, way too early, or way too late.
The system takes care of the rest.
However, read on if you want the full details!

### PID Controllers!

Tempo uses a PID controller to control scheduling.

Basically, a PID controller is a way to adjust a value in a system so that it approaches a target quickly and accurately.
On every iteration, it measures how far off the current value is from the target to determine how large of an adjustment to make.
The PID stands for the three factors in the calculation: Proportional, Integral, and Derivative.

- **Proportional** is just the amount the current value and the target value differ (from now on we'll call this the error.)
  Basically, this means that if we're far from the target we'll make a big correction.

- **Integral** is the sum of all the errors we've seen so far.
  Summing the errors means that the amount we adjust will grow over time if we're consistently off in one direction.

- **Derivative** is the rate of change in error over time (practically speaking, it's the current error minus the last error.)
  This is used as a dampening factor: if we have a bunch of small errors and then suddenly a HUGE one, reacting too quickly would mean potentially overshooting the target.
  By paying attention to the rate of change, we can avoid making sudden, unpredicatable adjustments.

The final adjustment combimes these along the lines of `p + i - d`, where each component is also multiplied by a weight.
(However, ignoring the derviative is pretty common; that's known as a PI controller.)

Despite being pretty simple, PID controllers are ubiquitous: they're probably used in your thermostat, your car's cruise control, your electric kettle, and so on.
Any time there's a controller that needs to make adjustments to match a target, a PID controller is probably involved somehow.

### So how does this apply to Tempo?

So we've seen that a PID controller needs to have both a target and current value to work.
But Tempo deals with *finding* the target value, so how does that work?

Basically, instead of subtracting the current value from the target value to derive an error, we just ask you for feedback!

A "too early" or "too late" might be an error value of something like 1 day, where "way to early" or "way to late" may be like 3 days.
Because of the way we weight the PID components, small adjustments over time result in approaching the optimum cadence (or at least that's the idea.)

Tempo also makes a small change to the normal PID controller logic: evaluation steps are infrequent enough that it'll be hard to notice a small drift over a long time, which the integral component tends to produce.
Because of this, we halve the integral component whenever you say that the cadence was about right.
This means that over time, it'll adjust less and less to keep the item in the right place.

## Acknowledgements

I took lots of inspiration for this tool from [Andy Matuschak's notes on spaced repetition](https://notes.andymatuschak.org/z2gqazXUkf9qyFjMQg4W3dw6yegnAJszvDywN).

## License

Tempo is licensed under [The Hippocratic License](https://firstdonoharm.dev), version 3.0.
See LICENSE in the source for the applicable modules to this project.
