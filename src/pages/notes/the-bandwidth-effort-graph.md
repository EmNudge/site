---
pubDate: Aug 1, 2026
title: "The bandwidth effort graph"
summary: how to allocate your time
tags: general
layout: ../../layouts/Blog.astro
---

In any moderately effective company, it is not effort that is rewarded, but impact. 

Late hours and stressful mornings do not on their own move the needle. You could be working hard on the wrong problem. 

It's exceptionally difficult to measure impact in any objective way. Lots of companies fall back on "vibes" for a lot of their promotions and performance packet analysis. But a good company will at least attempt to orient these vibes in some direction. 

"He's a hard worker, and dammit he gets results," you'll hear them say. Or perhaps, "he's not the hardest worker, but dammit he gets results". You'll rarely hear "he's a hard worker, and dammit he gets no results". 

To someone high enough on the totem pole, the effort of their lowest-paid worker is least visible compared with the impact of their work. An executive won't hear about the late nights by a young analyst, but might hear about who or what moved the company needle into the green or red.

If your goal is to either keep your job or gain promotions and you work at a company with some attention to the impact of your work, you'd be wise to chase impact, not effort.

# The point plot

If tasks presented to you are plotted on some graph where the y-axis is available bandwidth and the x-axis is available impact, you'd be best served to take projects from the lower-right-most portion.

<pre class="mermaid">
quadrantChart
  x-axis Low Impact --> High Impact
  y-axis Low Bandwidth --> High Bandwidth
  Task A: [0.12, 0.72]
  Task B: [0.25, 0.55]
  Task C: [0.38, 0.82]
  Task D: [0.30, 0.20]
  Task E: [0.48, 0.42]
  Task F: [0.55, 0.65]
  Task G: [0.62, 0.28]
  Task H: [0.72, 0.85]
  Task I: [0.78, 0.35]
  Task J: [0.88, 0.18]
</pre>

The one missing piece from this chart is that bandwidth is the input, not just an attribute of the task. It's a finite resource, so you can choose a limited total number of these projects.

You'll have a very rough sense of your capacity, but nothing as exact as a percentage. Still, I really like the framing from [Doing nothing at work](https://www.seangoedecke.com/doing-nothing-at-work/) where Sean advocates running (at most) around 80% of total available bandwidth. 

So you have a finite resource with 20% already chopped off. Running at 100% is dangerous (and should be reserved for high urgency infrequent moments), so you've got 80% of it left to play a maximal-packing algorithm on your remaining tasks.

<pre class="mermaid">
---
config:
  themeVariables:
    xyChart:
      backgroundColor: transparent
      plotColorPalette: "#6c6f7f, #4fc1f1, #7bd88f, #e5b567, #c586c0"
---
xychart-beta
  title "Packing tasks into remaining bandwidth"
  x-axis ["Bandwidth", "Packed tasks"]
  y-axis 0 --> 100
  bar [100, 0]
  bar [80, 0]
  bar [0, 80]
  bar [0, 45]
  bar [0, 20]
</pre>

Doing this is difficult, especially if taking on more tasks with your available bandwidth results in maximizing your capacity. Both projects might pick up steam with deadlines set outside your control. Without additional resources, you'll begin to allocate too much of your time (possibly more than is available) to both tasks.

A good recipe for burnout, failure, or both.

# Choosing projects

So here's what I do. At any moment, I'll have around 3 projects cooking. 

Firstly, there's the highest-visibility task (we're always playing politics to some degree). This task should be either medium or high impact. The highest-impact work is often not the most visible, so a medium-impact item usually lands here. You should refuse a low-impact task, if possible. Argue against it. These are landmines. 

Say this took about 50% of your bandwidth. You have 30% left.

I'll dedicate 15% to a long shot and 15% to a fun project. Both should be low visibility, without many external stakeholders.

<pre class="mermaid">
pie
  "Highest-visibility task": 50
  "Long shot": 15
  "Fun project": 15
  "Leisure": 20
</pre> 

Long shots are really helpful to establish surprising impact patterns and make a name for yourself if they pan out. If they don't, you lost nothing since no one was impacted by it not shipping. 

Fun projects are to keep me sane and engaged. They increase my energy instead of depleting it. We're squishy machines, and so my "total capacity" analogy is somewhat misplaced. This resource is finite, but not strictly predetermined or allocated.

I dedicate the remaining 20% purely to leisure. If I'm in office, I'll chat with coworkers or read articles. I'll tune my personal productivity programs (which really take more time setting up than they give back). 

This 20% is a freely distributable resource, so it can be allocated to whatever project is more urgent or engaging at that moment.

# The final model

You have a resource called bandwidth. This is a squishy resource that can be loosely determined ahead of time.

The company recognizes an attribute called impact. This is what most of your dedication should end with.

Politics force some play with optics. You can't be visibly free for half the time you're paid for. It's also not a smart setup. 

The available tasks will dictate your final allocation, but remember to pre-allocate a rough 20% as an emergency fund.

With the remaining 80%, find one medium-to-high-impact project to pin on. Any remaining bandwidth within this bucket should go to engaging work and long-shots, both hopefully low visibility so that their disruption does not cause trouble.
