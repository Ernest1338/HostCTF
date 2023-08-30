<script lang="ts">
	import { isLogged } from '$lib';

	let isLoggedValue = false;
	isLogged.subscribe((value: boolean) => {
		isLoggedValue = value;
	});
</script>

<nav>
	<a href="/">Home</a>
	<a href="/challenges">Challenges</a>
	<a href="/scoreboard">Scoreboard</a>
	{#if isLoggedValue}
		<a href="/profile">Profile</a>
		<a href="/logout">Logout</a>
	{:else}
		<a href="/register">Register</a>
		<a href="/login">Login</a>
	{/if}
</nav>
