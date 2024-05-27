---
pubDate: May 26, 2024
title: "Simple Global State In React"
summary: Jotai in 20 lines
tags: coding
layout: ../../layouts/Blog.astro
---

I've been explaining how libraries like [Jotai](https://jotai.org/) work fairly often recently. They provide a "signal-like" experience in React by exposing "subscribe-able" state. Jotai calls it "atoms". You can call it whatever.

## Signal-based Global State

The pattern is this.
```js title="state.js"
export const usernameSignal = signal('John');
```

```jsx title="components.jsx"
import { useSignal } from 'library';
import { usernameSignal } from './state.js';

export const UserIcon = () => {
	const [username, setUsername] = useSignal(usernameSignal);
	return <div>{username}</div>;
}
export const UserPanel = () => {
	const [username, setUsername] = useSignal(usernameSignal);
	return <section>{username}</section>;
}
```

When calling `setUsername` in any component, it will update all components that make use of `username`. A call of `setUsername` in our `UserPanel` component will also update the `username` in our `UserIcon` component.

How?

With a hidden mapping of signal objects to a list of `setState` functions.

# The Library

It may be clearer with this stripped-down example of the implementation.

```js title="library.js"
import { useState } from 'react';

const setStateFuncs = new Set();
export const useSignal = (signal) => {
	const [state, setState] = useState(signal.value); 
	setStateFuncs.add(setState);
	
	const updateAllStates = (newVal) => {
		for (const setStateFunc of setStateFuncs) {
			setStateFunc(newVal);
		}
	};

	return [state, updateAllStates];
}
```

When we call our local `setState`, we're actually calling all `setState` functions from anyone that is also using this signal. The `setState` it calls will update the component that is using this signal.

For this to actually be useful, however, we'll need to actually map our signal object to its own `setStateFuncs` list, making our full code look more like this.

```js title="library.js"
import { useState } from 'react';

const setStateMap = new Map();

export const useSignal = (signal) => {
	// get the corresponding list of setState funcs for this signal
	const setStateFuncs = setStateMap.get(stateObj) ?? new Set();
	setStateMap.set(stateObj, setStateFuncs);
  
	const [state, setState] = useState(signal.value); 
	setStateFuncs.add(setState);
	
	const updateAllStates = (newVal) => {
		for (const setStateFunc of setStateFuncs) {
			setStateFunc(newVal);
		}
	};

	return [state, updateAllStates];
}
```

Because our "signal objects" just need to be an object with a value property, they can be defined as simply as:
```js title="state.js"
export const usernameSignal = { value: 'John' };
```

The full code (with some minor tweaks) is available as a [github gist here](https://gist.github.com/EmNudge/7092b70635c6067c8759f55e75d1c9f3).

\# end note