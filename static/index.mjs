const ws = new WebSocket('ws://127.0.0.1:6969/sync');
ws.addEventListener('message', (event) => {
    const { cpus, ram } = JSON.parse(event.data);
    let drawn = false;

    const cpus_el = document.querySelector('#cpus');

    // Clear UI components
    cpus_el.innerHTML = '';

    // Update the UI for CPU
    for (let cpu of cpus) {
        if (!drawn) {
            const el = document.createElement('div');
            el.classList.add('cpu_unit');

            const fullbar = document.createElement('div');
            fullbar.classList.add('fullbar');
            const span = document.createElement('span');
            span.textContent = cpu + '%';
            fullbar.appendChild(span);

            const bar = document.createElement('div');
            bar.classList.add('bar');
            bar.style.width = cpu + '%';

            el.appendChild(fullbar);
            el.appendChild(bar);
            cpus_el.appendChild(el);
        } else {
            const cpu_els = document.querySelectorAll('.cpu_unit');
            // Update the UI for CPU
            for (let cpu_el of cpu_els) {
                const bar = cpu_el.querySelector('.bar');
                const span = cpu_el.querySelector('span');
                bar.style.width = cpu + '%';
                span.textContent = cpu + '%';
            }
        }
    }

    // Update the UI for RAM
    const ram_gb = ram.map((ram) => ram / 1024 / 1024 / 1024);
    const ram_percentage = ram_gb[0] / ram_gb[1] * 100;

    const ram_el = document.querySelector('#ram');
    const span = ram_el.querySelector('span');
    span.textContent = ram_gb[0] + 'GB / ' + ram_gb[1] + 'GB (' + ram_percentage + '%)';
    const ram_bar = ram_el.querySelector('.ram-bar');
    console.log(ram_bar);
    ram_bar.style.width = ram_percentage + '%';


    drawn = true;
});
