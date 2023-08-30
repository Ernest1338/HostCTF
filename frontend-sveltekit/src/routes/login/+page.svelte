<script lang="ts">
	import { onMount } from 'svelte';
	onMount(() => {
		document.addEventListener('keydown', (event: KeyboardEvent) => {
			if (event.key == 'Enter') {
				if (location.pathname == '/login') {
					login();
				}
			}
		});
	});

	import { BACKEND_URL } from '../../config';
	import Infobox, { showInfo } from '../../components/infobox.svelte';
	import { setCookie, setCookieArray, isLogged } from '$lib';

	let username = '';
	let password = '';
	let submitting = false;

	async function login() {
		if (submitting) {
			return;
		}
		submitting = true;
		const user = {
			username: username,
			password: password
		};
		const response = await fetch(BACKEND_URL + '/login', {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(user)
		});
		const response_json = await response.json();

		if (response_json['status'] == 'OK') {
			username = '';
			password = '';
			showInfo('success', 'Login successful!');
			// recreate solved_chals cookie
			setCookieArray('solved_chals', response_json['solved_chals']);
			// set auth cookie
			setCookie('auth_key', response_json['auth_key']);
			// set 'logged_as' cookie
			setCookie('logged_as', user.username);
			isLogged.set(true);
		} else {
			showInfo('warning', response_json['cause']);
			setTimeout(function () {
				submitting = false;
			}, 1000);
		}
	}
</script>

<Infobox />
<h1>Login</h1>
<form>
	<label for="username">Username</label>
	<input type="text" placeholder="Username" id="username" bind:value={username} /><br />

	<label for="password">Password</label>
	<input type="password" placeholder="Password" id="password" bind:value={password} /><br />

	<input type="button" value="Submit" disabled={submitting} on:click={login} />
</form>
