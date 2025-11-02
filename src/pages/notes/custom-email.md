---
pubDate: June 15, 2025
title: "I don't pay for a custom email address"
summary: but I have a custom email
tags: coding
layout: ../../layouts/Blog.astro
---

I have a custom email. You can find it on the bottom of this site, protected with some JS (i.e. invisible for very basic web crawlers).

This seemed important to have when I set it up. I don't get a ton of use out of it, but it gives people a way to contact me when my blog posts circulate in some spaces.

Instead of paying some nominal sum for an email inbox, I pay $0 with email routing and aliasing. 

# Here is how

First, set up email forwarding. This is pretty straightforward with DNS-level support, which is pretty common. I use CloudFlare which lets me set up email forwarding rules. Any emails to `myhandle@website.com` get sent to my personal address.

This might be enough for your use case, but it helps to be able to send them with the same address. For this I add a custom "account" to gmail (under the "*Accounts and Import*" tab of the settings page). 

On the first page it asks for the "send as" email (the one we're routing) and the handle you want. Here's what I might put into it:
- **Name**: My Custom Handle
- **Email Address**: `myhandle@website.com`

On the next, it will ask for some more technical information. Assuming I'm routing `myhandle@website.com` to my personal gmail account `my.name@gmail.com`, the values might look like this:

- **SMTP Server**: `smtp.gmail.com`
- **Port**: 465
- **Username**: `my.name@gmail.com`
- **Password**: `<Google App Password>`

That Google App Password must be created through the normal way. It's somewhere under the "Sign-in & security" section of your google account, but there are enough guides for this already

