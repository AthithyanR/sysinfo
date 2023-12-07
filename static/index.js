async function update(cpus) {
    const cpuTemplate = `
        <div class="template">
        ${cpus.map((cpu, idx) => `
            <div class="cell-wrapper">
                <div>
                    core #${idx}: 
                </div>
                <div style="width:${400 * (cpu / 100)}px">
                    ${cpu.toFixed(2)}%
                </div>
            </div>
        `).join('')}
        </div>
    `;

    document.body.innerHTML = cpuTemplate;
};

function on_load() {
    const ws = new WebSocket(`ws://${location.host}/realtime/cpus`);
    ws.onmessage = (event) => {
        const cpus = JSON.parse(event.data);
        update(cpus);
    };
    ws.onerror = console.error;
}

document.addEventListener('DOMContentLoaded', on_load);