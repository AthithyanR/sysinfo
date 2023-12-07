async function update() {
    try {
        const res = await fetch('/api/cpus');
        const cpus = await res.json();

        const cpuTemplate = [
            '<div>',
            ...cpus.map((cpu, idx) => `<div>core #${idx}: ${cpu}</div>`),
            '</div>'
        ].join('');

        document.body.innerHTML = cpuTemplate;
    } catch (e) {
        console.log(e);
    }
};

function on_load() {
    setInterval(update, 1000);
}

document.addEventListener('DOMContentLoaded', on_load);