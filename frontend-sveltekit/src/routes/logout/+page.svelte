<script lang="ts">
	import Infobox, { showInfo } from '../../components/infobox.svelte';
	import { isLogged } from '$lib';
	import Page from '../+page.svelte';

	const cookies = document.cookie.split(';');

	for (let i = 0; i < cookies.length; i++) {
		const cookie = cookies[i];
		const eqPos = cookie.indexOf('=');
		const name = eqPos > -1 ? cookie.substring(0, eqPos) : cookie;
		document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT';
	}

	isLogged.set(false);
	showInfo('success', 'Logged out');
</script>

<Infobox />
<Page />
