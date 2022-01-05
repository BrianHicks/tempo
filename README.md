# Tempo

Tempo helps you keep up with the things that matter to you by keeping track of things and listening when you tell it that something was scheduled too early or too late.

The goal, basically, is to help you keep up with things with a minimum of effort on your part.
Using Tempo should feel joyful and effortless.

Let's have some examples:

- Have a favorite journaling prompt?
  Add it to the system and it'll try and find the right time for you to come back to it.
  Or, if it turns out you don't like the prompt, it'll make sure you don't see it for a long time.

- Let's develop some ideas!
  Look at your WIP writing every so often; Tempo will schedule sooner for productive writing sessions and defer ideas that still need time to marinate.

- How do you save bookmarks and references?
  Grab a handful and review them every day.
  If they're useful, they can show up more often.
  If they're not, they'll get scheduled far into the future (just in case.)

- How about balancing side projects?
  Active projects will show up sooner, while inactive projects will be deferred but not forgotten.

- Stay in touch with other people: add journaling prompts ("Think about a nice time you had with Marcus") or contact prompts ("Have you talked with Julie recently? Maybe it'd be good to text her.") or something else entirely!

- Want to stay up on interesting topics?
  Prompt yourself: "find developments about […]", or just "go check this site for interesting things about […]".

## How Does it Work?

Every item in Tempo has a cadence and a next due date (you can specify both of these when adding an item, and guesses are fine!)
Over time, Tempo will use a PID controller to refine the cadence based on how far off you say the last guess was.

Generally, you don't have to worry about the technical details: you just say if something showed up too early, too late, way too early, or way too late.
The system takes care of the rest.
however, read on if you want the full details!

### Wait, what's a PID controller?

Basically: it's a way to adjust a value in a system so that it approaches a target quickly and accurately.
It uses measurements of how far off the current value is from the target to determine how large of an adjustment to make.
The PID stands for the three factors in the calculation: Proportional, Integral, and Derivative.

- **Proportional** is just the amount the current value and the target value differ (from now on we'll call this the error.)
  Basically, this means that if we're far from the target we'll make a big correction.

- **Integral** is the sum of all the errors we've seen so far.
  Summing the errors means that if the error is large over time, the adjustment will be larger over time.

- **Derivative** is the rate of change in error over time (practically speaking, it's the current error minus the last error.)
  This is used as a dampening factor: we don't want to move too quickly towards our target value and end up overshooting it by a lot, so the faster the error is changing the more dampening we'll apply.

The final adjustment is usually something like `p + i - d`, where each component is also multiplied by a weight.
However, ignoring the derviative is pretty common (that's a PI controller.)

Despite being pretty simple, PID controllers are ubiquitous: they're probably used in your thermostat, your car's cruise control, your electric kettle, and so on.
Any time there's a controller that needs to make adjustments to match a target, a PID controller is probably involved somehow.

### So how does this apply to Tempo?

So we've seen that a PID controller needs to have both a target and current value to work, right?
But Tempo deals with *finding* the target value, so how does that work?

Basically, instead of subtracting the current value from the target value to get an error, we just ask you!

A "too early" or "too late" might be an error value of something like 1 day, where "way to early" or "way to late" may be like 3 days.
Because of the way we weight the PID components, small adjustments over time result in approaching the optimum cadence (or at least that's the idea.)
