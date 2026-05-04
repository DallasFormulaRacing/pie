registry := "ghcr.io/schommie"

# Cross-build both images for arm64
build:
    @bash -u -o pipefail -c 'status=0; just build-backend & backend=$!; just build-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'

build-backend:
    podman build --platform linux/arm64 -t {{registry}}/ev-can-bridge:latest ./backend

build-gui:
    podman build --platform linux/arm64 -t {{registry}}/ev-gui:latest ./gui

# Push images to GHCR
push:
    @bash -u -o pipefail -c 'status=0; just push-backend & backend=$!; just push-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'

push-backend:
    podman push {{registry}}/ev-can-bridge:latest

push-gui:
    podman push {{registry}}/ev-gui:latest

# Build and push (run on dev machine)
deploy:
    @bash -u -o pipefail -c 'status=0; just build-backend & backend=$!; just build-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'
    @bash -u -o pipefail -c 'status=0; just push-backend & backend=$!; just push-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'
    @echo "Done. Run 'sudo podman auto-update' on the Raspi."

# Start dev environment
dev:
    podman-compose -f compose.dev.yml up --build

# Start prod stack
up:
    podman-compose up -d

# Stop all services
down:
    podman-compose down

login:
    sudo podman login {{registry}} --username schommie --password-stdin < ~/.ghtoken

# Capture websocket broadcast messages to log.txt for N seconds
ws-log seconds:
    @node -e 'const fs=require("fs"); const seconds=Number(process.argv[1]); if(!Number.isFinite(seconds)||seconds<=0){console.error("usage: just ws-log <seconds>"); process.exit(2)} const url=process.env.BACKEND_WS_URL||"ws://pi.local:9002"; const rows=[]; const ws=new WebSocket(url); let done=false; const finish=(code=0)=>{if(done)return; done=true; fs.writeFileSync("log.txt", rows.join("\n")+(rows.length?"\n":"")); console.error(`wrote ${rows.length} websocket messages to log.txt from ${url}`); process.exit(code)}; const timer=setTimeout(()=>{try{ws.close()}catch{} finish(0)}, seconds*1000); ws.addEventListener("open",()=>rows.push(JSON.stringify({capturedAt:new Date().toISOString(),event:"open",url}))); ws.addEventListener("message",ev=>{let data; try{data=JSON.parse(ev.data)}catch{data=ev.data} rows.push(JSON.stringify({capturedAt:new Date().toISOString(),data}))}); ws.addEventListener("error",err=>{clearTimeout(timer); rows.push(JSON.stringify({capturedAt:new Date().toISOString(),event:"error",message:String(err.message||err)})); finish(1)}); ws.addEventListener("close",()=>{clearTimeout(timer); finish(0)});' {{seconds}}
