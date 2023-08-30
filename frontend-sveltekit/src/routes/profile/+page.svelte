<script lang="ts">
	import { getCookie } from '$lib';
	import Infobox, { showInfo } from '../../components/infobox.svelte';
	import { BACKEND_URL } from '../../config';

	let isLogged = document.cookie.includes('logged_as');
	let username = '';
	let score = '-';

	if (isLogged) {
		updateProfile();
	}

	async function updateProfile() {
		username = getCookie('logged_as');
		const response = await fetch(BACKEND_URL + '/profile', {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ username: username })
		});
		const response_json = await response.json();

		if (response_json['status'] == 'OK') {
			score = response_json['score'];
		} else {
			showInfo('warning', response_json['cause']);
		}
	}
</script>

<article>
	<h2 style="text-align: center;">Profile</h2>
</article>
<Infobox />
{#if isLogged}
	<article style="background-color: var(--base);">
		<h2 style="text-align: center;">{username}</h2>
		<p style="font-size: 1.5em;">Score: {score}</p>
	</article>
{:else}
	<article style="background-color: var(--base);">
		<h3 style="text-align: center;">You need to be logged in to view this page!</h3>
	</article>
{/if}
