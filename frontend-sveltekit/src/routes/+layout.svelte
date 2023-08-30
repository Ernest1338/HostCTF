<script>
	import Navbar from '../components/navbar.svelte';
</script>

<header>
	<h1>HostCTF - Hacking Competition</h1>
	<Navbar />
</header>
<main>
	<slot />
</main>
<footer>
	<p>
		HostCTF - Created by Ernest Gupik (<a href="https://eg-zine.cf/">eg-zine.cf</a>). Powered by
		<a href="https://github.com/Ernest1338/HostCTF" target="_blank">HostCTF</a> (self made CTF platform).
	</p>
</footer>
