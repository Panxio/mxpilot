<script>
  import { selectedCid, uiState, statusData, BUTTON_DEFINITIONS } from './stores.js';

  function selectButton(cid) {
    selectedCid.set(cid);
  }

  function handleKeyDown(e, cid) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      selectButton(cid);
    }
  }

  function getButtonActionName(cid, mapping) {
    if (!mapping || !mapping[cid]) return 'Por defecto del sistema';
    const act = mapping[cid];
    if (act.type === 'SingleKey') {
      if (act.key === 'KEY_LEFTMETA' || act.key === 'SUPER' || act.key === 'META') return 'Super (Actividades)';
      if (act.key === 'KEY_MUTE') return 'Silenciar';
      if (act.key === 'KEY_PLAYPAUSE') return 'Play / Pausa';
      return act.key;
    }
    if (act.type === 'KeyCombo') {
      return `${act.modifiers.join('+')}+${act.key}`.replace(/KEY_/g, '');
    }
    if (act.type === 'MouseButton') return act.button.replace('BTN_', 'Botón ');
    if (act.type === 'None') return 'Deshabilitado';
    return 'Personalizado';
  }
</script>

<div class="relative flex flex-col items-center p-4 bg-slate-900/40 rounded-2xl border border-slate-800/80 shadow-inner backdrop-blur-sm h-full min-h-0 overflow-y-auto">
  <!-- Encabezado simétrico con la columna derecha -->
  <div class="w-full flex items-center justify-between pb-3 border-b border-slate-800/60 mb-2 flex-shrink-0">
    <div class="flex items-center space-x-2">
      <span class="w-2 h-2 rounded-full {$uiState === 'Connected' ? 'bg-emerald-500 shadow-sm shadow-emerald-500/50' : 'bg-amber-500'}"></span>
      <h2 class="text-sm font-semibold text-white tracking-wide">
        MX Vertical <span class="text-xs font-normal text-slate-400 font-mono">(57° Ergo)</span>
      </h2>
    </div>
    {#if $uiState !== 'Connected'}
      <span class="inline-flex items-center whitespace-nowrap px-2.5 py-1 rounded-full text-[11px] font-medium bg-amber-500/10 border border-amber-500/30 text-amber-400">
        {$uiState === 'DaemonDown' ? 'Servicio Inactivo' : 'Desconectado'}
      </span>
    {:else}
      <span class="inline-flex items-center whitespace-nowrap px-2.5 py-1 rounded-full text-[11px] font-medium bg-slate-800/90 border border-slate-700/80 text-slate-400 font-mono">
        Blueprint Interactivo
      </span>
    {/if}
  </div>

  <!-- Contenedor del SVG Blueprint levantado y proporcionado -->
  <div class="w-full max-w-[360px] aspect-[360/300] my-2 flex items-center justify-center transition-all duration-300 flex-shrink-0 {$uiState !== 'Connected' ? 'opacity-40 grayscale-[40%]' : ''}">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 320" width="100%" height="100%" class="overflow-visible select-none">
      <defs>
        <!-- Sombra ambiental suave -->
        <radialGradient id="floor-shadow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="#020617" stop-opacity="0.7" />
          <stop offset="65%" stop-color="#020617" stop-opacity="0.25" />
          <stop offset="100%" stop-color="#020617" stop-opacity="0" />
        </radialGradient>

        <!-- Cuerpo principal (cavidad ergonómica del pulgar) -->
        <linearGradient id="body-grad" x1="15%" y1="15%" x2="85%" y2="85%">
          <stop offset="0%" stop-color="#334155" />
          <stop offset="38%" stop-color="#1e293b" />
          <stop offset="80%" stop-color="#0f172a" />
          <stop offset="100%" stop-color="#090d16" />
        </linearGradient>

        <!-- Faldón / Base inferior -->
        <linearGradient id="skirt-grad" x1="0%" y1="0%" x2="100%" y2="40%">
          <stop offset="0%" stop-color="#1e293b" />
          <stop offset="45%" stop-color="#0f172a" />
          <stop offset="100%" stop-color="#060911" />
        </linearGradient>

        <!-- Botones principales en escorzo (perfil izquierdo) -->
        <linearGradient id="island-grad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#475569" />
          <stop offset="50%" stop-color="#263346" />
          <stop offset="100%" stop-color="#0f172a" />
        </linearGradient>

        <!-- Placa superior de aluminio anodizado -->
        <linearGradient id="metal-top" x1="0%" y1="100%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="#94a3b8" />
          <stop offset="45%" stop-color="#cbd5e1" />
          <stop offset="80%" stop-color="#f1f5f9" />
          <stop offset="100%" stop-color="#94a3b8" />
        </linearGradient>

        <!-- Bisel lateral de la placa de aluminio -->
        <linearGradient id="metal-side" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="#475569" />
          <stop offset="30%" stop-color="#cbd5e1" />
          <stop offset="65%" stop-color="#94a3b8" />
          <stop offset="100%" stop-color="#334155" />
        </linearGradient>

        <!-- Gradiente interactivo estándar para botones -->
        <linearGradient id="interactive-btn" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#1e293b" />
          <stop offset="100%" stop-color="#0f172a" />
        </linearGradient>

        <!-- Gradiente botón activo seleccionado (cyan vibrante) -->
        <linearGradient id="selectedGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#38bdf8" />
          <stop offset="100%" stop-color="#0284c7" />
        </linearGradient>

        <!-- Gradiente para las estrías de goma del pulgar -->
        <linearGradient id="ridge-grad" x1="0%" y1="0%" x2="25%" y2="100%">
          <stop offset="0%" stop-color="#64748b" stop-opacity="0.1" />
          <stop offset="45%" stop-color="#94a3b8" stop-opacity="0.38" />
          <stop offset="85%" stop-color="#475569" stop-opacity="0.22" />
          <stop offset="100%" stop-color="#0f172a" stop-opacity="0" />
        </linearGradient>

        <!-- Sombra suave para dar relieve a las estrías -->
        <linearGradient id="ridge-dark" x1="0%" y1="0%" x2="25%" y2="100%">
          <stop offset="0%" stop-color="#020617" stop-opacity="0" />
          <stop offset="45%" stop-color="#020617" stop-opacity="0.55" />
          <stop offset="85%" stop-color="#020617" stop-opacity="0.35" />
          <stop offset="100%" stop-color="#020617" stop-opacity="0" />
        </linearGradient>

        <!-- Resplandor sutil para elementos interactivos -->
        <filter id="interactive-glow" x="-20%" y="-20%" width="140%" height="140%">
          <feDropShadow dx="0" dy="0" stdDeviation="2" flood-color="#60a5fa" flood-opacity="0.35" />
        </filter>

        <!-- Resplandor intenso de selección activa -->
        <filter id="selectedGlow" x="-40%" y="-40%" width="180%" height="180%">
          <feDropShadow dx="0" dy="0" stdDeviation="4.5" flood-color="#38bdf8" flood-opacity="0.85" />
        </filter>
      </defs>

      <!-- Sombra de apoyo sobre la superficie -->
      <ellipse cx="185" cy="276" rx="142" ry="12" fill="url(#floor-shadow)" />

      <!-- ==================== 1. RUEDA DE SCROLL (CID 0x0052) ==================== -->
      <g class="cursor-pointer group" on:click={() => selectButton(0x0052)} on:keydown={(e) => handleKeyDown(e, 0x0052)} role="button" tabindex="0" aria-label="Rueda Central Scroll">
        <g id="svg-btn-wheel" filter={$selectedCid === 0x0052 ? 'url(#selectedGlow)' : 'url(#interactive-glow)'} class="transition-all duration-200 group-hover:brightness-125">
          <!-- Arco compacto que asoma discretamente por el perfil izquierdo -->
          <path d="M 97,134 
                   C 92,129 98,113 107,113 
                   L 113,117 
                   L 102,138 
                   Z" 
                fill={$selectedCid === 0x0052 ? 'url(#selectedGrad)' : 'url(#interactive-btn)'} 
                stroke={$selectedCid === 0x0052 ? '#38bdf8' : '#60a5fa'} 
                stroke-width={$selectedCid === 0x0052 ? 2.2 : 1.4} 
                stroke-linejoin="round" />
          <!-- Textura ranurada fina del scroll -->
          <path d="M 96,129 L 101,132 
                   M 96.5,125 L 102,128 
                   M 98,121 L 103.5,124 
                   M 100.5,117.5 L 106,120.5 
                   M 104,115 L 108.5,117.5" 
                stroke={$selectedCid === 0x0052 ? '#e0f2fe' : '#818cf8'} 
                stroke-width="0.95" stroke-linecap="round" opacity="0.85" />
          <!-- Borde interior luminoso -->
          <path d="M 97.5,132 C 93.8,128 99,115 106.5,114.5" fill="none" stroke="#93c5fd" stroke-width="0.75" opacity="0.8" stroke-linecap="round" />
        </g>
      </g>

      <!-- ==================== 2. RAMPA FRONTAL (ÁNGULO DE 57°) ==================== -->
      <g id="front-button-island">
        <path d="M 136,68 C 112,99 88,142 69,193 L 89,192 C 107,145 129,103 151,72 Z" fill="url(#island-grad)" stroke="#475569" stroke-width="1" stroke-linejoin="round" />
        <!-- Borde de luz en el filo frontal de 57 grados -->
        <path d="M 136,69 C 112,100 88,142 70,192" fill="none" stroke="#94a3b8" stroke-width="1" opacity="0.45" />
        <!-- Hendidura divisoria entre los dos clics principales -->
        <path d="M 94,137 C 100,142 106,146 112,149" fill="none" stroke="#090d16" stroke-width="2" stroke-linecap="round" />
        <path d="M 93.5,138.6 C 99.5,143.6 105.5,147.6 111.5,150.6" fill="none" stroke="#64748b" stroke-width="0.75" opacity="0.55" stroke-linecap="round" />
      </g>

      <!-- ==================== 3. CHASIS PRINCIPAL Y ALA DEL PULGAR ==================== -->
      <g id="main-chassis">
        <!-- Escultura principal con caída cóncava para el reposo del pulgar -->
        <path d="M 151,72 
                 L 164,77 
                 C 170,80 177,80 184,77 
                 L 234,54 
                 C 242,96 259,154 312,188 
                 C 314,190 315,192 314,194 
                 L 86,238 
                 L 56,238 
                 C 59,221 63,207 69,193 
                 L 89,192 
                 C 107,145 129,103 151,72 Z" fill="url(#body-grad)" stroke="#334155" stroke-width="1" stroke-linejoin="round" />

        <!-- Sombra suave de transición en la vaguada del pulgar -->
        <path d="M 176,82 C 185,138 215,185 308,192 L 150,224 C 155,175 162,120 176,82 Z" fill="#090d16" opacity="0.28" />

        <!-- Línea de partición técnica entre la isla frontal y el cuerpo -->
        <path d="M 151,72 C 129,103 107,145 89,192 L 69,193" fill="none" stroke="#090d16" stroke-width="1.6" stroke-linejoin="round" />

        <!-- Costura diagonal interna que baja hasta el vértice del puerto USB-C -->
        <path d="M 89,192 L 75,228" fill="none" stroke="#090d16" stroke-width="1.3" />
        <path d="M 90,192 L 76,228" fill="none" stroke="#475569" stroke-width="0.6" opacity="0.4" />
      </g>

      <!-- ==================== 4. ESTRÍAS DE AGARRE DEL PULGAR ==================== -->
      <g id="svg-thumb-ridges" fill="none" stroke-linecap="round">
        <!-- Sombra de las estrías (relieve negativo) -->
        <g stroke="url(#ridge-dark)" stroke-width="2.2">
          <path d="M 229,63 C 237,108 258,156 303,189" />
          <path d="M 224,66 C 231,112 251,160 293,192" />
          <path d="M 218,70 C 224,116 242,163 282,195" />
          <path d="M 212,73 C 217,120 233,166 270,197" />
          <path d="M 205,76 C 209,123 223,169 257,200" />
          <path d="M 198,80 C 201,126 212,172 243,203" />
          <path d="M 190,83 C 191,129 200,175 228,206" />
          <path d="M 181,86 C 181,132 187,178 212,209" />
        </g>
        <!-- Luz de las estrías (relieve positivo) -->
        <g stroke="url(#ridge-grad)" stroke-width="1.1">
          <path d="M 230,63 C 238,108 259,156 304,189" />
          <path d="M 225,66 C 232,112 252,160 294,192" />
          <path d="M 219,70 C 225,116 243,163 283,195" />
          <path d="M 213,73 C 218,120 234,166 271,197" />
          <path d="M 206,76 C 210,123 224,169 258,200" />
          <path d="M 199,80 C 202,126 213,172 244,203" />
          <path d="M 191,83 C 192,129 201,175 229,206" />
          <path d="M 182,86 C 182,132 188,178 213,209" />
        </g>
      </g>

      <!-- ==================== 5. FALDÓN BASE INFERIOR ==================== -->
      <g id="bottom-skirt">
        <!-- Patín PTFE / Base de deslizamiento inferior -->
        <path d="M 76,272 L 266,271 L 274,274 L 78,275 Z" fill="#020617" />

        <!-- Bloque sólido inferior que sostiene el reposapulgar -->
        <path d="M 56,238 
                 L 75,228 
                 L 314,193 
                 C 314,218 303,245 285,269 
                 C 282,272 278,273 272,273 
                 L 75,273 
                 C 68,273 64,269 61,262 
                 Z" fill="url(#skirt-grad)" stroke="#1e293b" stroke-width="1" stroke-linejoin="round" />

        <!-- Hendidura oscura de separación entre el cuerpo y la base -->
        <path d="M 56,238 L 75,228 L 314,193" fill="none" stroke="#050811" stroke-width="1.8" stroke-linejoin="round" />

        <!-- Reflejo especular en la arista viva del ala inferior -->
        <path d="M 56,239.5 L 75,229.5 L 314,194.5" fill="none" stroke="#64748b" stroke-width="0.8" opacity="0.55" stroke-linejoin="round" />
      </g>

      <!-- ==================== 6. PUERTO USB-C (FRONTAL INFERIOR) ==================== -->
      <g id="svg-usbc">
        <!-- Cavidad biselada en el faldón delantero -->
        <rect x="75" y="241" width="33" height="17" rx="7.5" fill="#060911" stroke="#334155" stroke-width="1" />
        <!-- Sombra profunda del conector -->
        <rect x="78" y="243.5" width="27" height="12" rx="5.5" fill="#020617" stroke="#1e293b" stroke-width="0.6" />
        <!-- Receptáculo metálico USB-C -->
        <rect x="81" y="245.5" width="21" height="8" rx="4" fill="#090d16" stroke="#60a5fa" stroke-width="1" opacity="0.9" />
        <!-- Lengüeta central de pines USB-C -->
        <line x1="85" y1="249.5" x2="98" y2="249.5" stroke="#93c5fd" stroke-width="1.6" stroke-linecap="round" />
      </g>

      <!-- ==================== 7. CORONA / PLACA METÁLICA SUPERIOR ==================== -->
      <g id="aluminum-crown">
        <!-- Sombra proyectada bajo el labio de aluminio -->
        <path d="M 136,69 L 164,79 C 170,82 177,82 184,79 L 234,56" fill="none" stroke="#020617" stroke-width="2.5" opacity="0.75" stroke-linecap="round" />

        <!-- Pared lateral biselada de la placa de aluminio -->
        <path d="M 140,59 
                 L 164,69 
                 C 170,71 176,71 182,68 
                 L 230,45 
                 L 234,54 
                 L 184,77 
                 C 177,80 170,80 164,77 
                 L 136,68 
                 C 136,64 138,61 140,59 Z" fill="url(#metal-side)" stroke="#64748b" stroke-width="0.8" stroke-linejoin="round" />

        <!-- Cara superior inclinada de aluminio cepillado -->
        <path d="M 140,59 
                 C 138,56 140,53 144,51 
                 L 196,26 
                 C 202,23 208,23 213,26 
                 L 228,37 
                 C 232,40 233,43 230,45 
                 L 182,68 
                 C 176,71 170,71 164,69 
                 Z" fill="url(#metal-top)" stroke="#e2e8f0" stroke-width="1" stroke-linejoin="round" />

        <!-- Brillo especular en la arista del bisel metálico -->
        <path d="M 141,59.5 L 164,68.5 C 170,70.5 176,70.5 182,67.5 L 229,44.5" fill="none" stroke="#ffffff" stroke-width="0.9" opacity="0.8" />
      </g>

      <!-- ==================== 8. BOTÓN SUPERIOR DPI (CID 0x00FD) ==================== -->
      <g class="cursor-pointer group" on:click={() => selectButton(0x00FD)} on:keydown={(e) => handleKeyDown(e, 0x00FD)} role="button" tabindex="0" aria-label="Botón Superior DPI">
        <g id="svg-btn-top" filter={$selectedCid === 0x00FD ? 'url(#selectedGlow)' : 'url(#interactive-glow)'} class="transition-all duration-200 group-hover:brightness-125">
          <!-- Bisel exterior encastrado en el aluminio -->
          <path d="M 197,36 
                   C 194,34 195,31 198,29 
                   L 204,26 
                   C 208,24 212,25 215,27 
                   L 223,33 
                   C 226,35 226,38 222,40 
                   L 215,43 
                   C 211,45 207,44 204,42 
                   Z" fill="#0f172a" stroke="#475569" stroke-width="0.8" />
          <!-- Superficie interactiva del botón de velocidad de cursor -->
          <path d="M 198.5,35.5 
                   C 196,34 196.5,31.5 199,30 
                   L 204.5,27.2 
                   C 208,25.5 211.5,26 214,28 
                   L 221.5,33.5 
                   C 224,35 224,37.5 221,39 
                   L 214.5,42 
                   C 211,43.5 207.5,43 205,41 
                   Z" 
                fill={$selectedCid === 0x00FD ? 'url(#selectedGrad)' : 'url(#interactive-btn)'} 
                stroke={$selectedCid === 0x00FD ? '#38bdf8' : '#60a5fa'} 
                stroke-width={$selectedCid === 0x00FD ? 2.4 : 1.6} 
                stroke-linejoin="round" />
          <!-- Destello técnico central -->
          <ellipse cx="210" cy="34.5" rx="4.5" ry="2.2" transform="rotate(-24 210 34.5)" 
                   fill={$selectedCid === 0x00FD ? '#38bdf8' : '#60a5fa'} 
                   opacity={$selectedCid === 0x00FD ? 0.6 : 0.25} />
        </g>
      </g>

      <!-- ==================== 9. CÁPSULA DE BOTONES LATERALES DEL PULGAR ==================== -->
      <!-- Alojamiento de la píldora bajo la cresta superior -->
      <path d="M 189,78 
               L 218,64 
               C 221,62.5 224.5,64 225,67.5 
               L 226,74 
               C 226.5,77.5 224.5,81 221,82.5 
               L 192,96.5 
               C 188.5,98 185.5,96 185,92.5 
               L 184,86 
               C 183.5,82.5 185.5,79.5 189,78 Z" fill="#060911" stroke="#334155" stroke-width="0.9" stroke-linejoin="round" />

      <!-- Botón Lateral Delantero - Forward (CID 0x0056) -->
      <g class="cursor-pointer group" on:click={() => selectButton(0x0056)} on:keydown={(e) => handleKeyDown(e, 0x0056)} role="button" tabindex="0" aria-label="Botón Lateral Adelante">
        <g id="svg-btn-forward" filter={$selectedCid === 0x0056 ? 'url(#selectedGlow)' : 'url(#interactive-glow)'} class="transition-all duration-200 group-hover:brightness-125">
          <path d="M 190,79.5 
                   L 204.5,72.5 
                   L 206,87.5 
                   L 192,94.5 
                   C 189,96 186.8,94.5 186.4,91.5 
                   L 185.6,86 
                   C 185.2,83 187,81 190,79.5 Z" 
                fill={$selectedCid === 0x0056 ? 'url(#selectedGrad)' : 'url(#interactive-btn)'} 
                stroke={$selectedCid === 0x0056 ? '#38bdf8' : '#60a5fa'} 
                stroke-width={$selectedCid === 0x0056 ? 2.2 : 1.5} 
                stroke-linejoin="round" />
          <!-- Relieve táctil sutil -->
          <path d="M 188,83.5 C 188,82 189.5,81 191,80.2 L 202.5,74.8" fill="none" stroke="#93c5fd" stroke-width="0.75" opacity="0.55" stroke-linecap="round" />
        </g>
      </g>

      <!-- Botón Lateral Trasero - Back (CID 0x0053) -->
      <g class="cursor-pointer group" on:click={() => selectButton(0x0053)} on:keydown={(e) => handleKeyDown(e, 0x0053)} role="button" tabindex="0" aria-label="Botón Lateral Atrás">
        <g id="svg-btn-back" filter={$selectedCid === 0x0053 ? 'url(#selectedGlow)' : 'url(#interactive-glow)'} class="transition-all duration-200 group-hover:brightness-125">
          <path d="M 206.5,71.5 
                   L 217.5,66 
                   C 220.5,64.5 222.8,66 223.2,69 
                   L 224,74.5 
                   C 224.4,77.5 222.5,80 219.5,81.5 
                   L 208,86.8 
                   Z" 
                fill={$selectedCid === 0x0053 ? 'url(#selectedGrad)' : 'url(#interactive-btn)'} 
                stroke={$selectedCid === 0x0053 ? '#38bdf8' : '#818cf8'} 
                stroke-width={$selectedCid === 0x0053 ? 2.2 : 1.5} 
                stroke-linejoin="round" />
          <!-- Relieve táctil sutil -->
          <path d="M 208.5,72.2 L 217.5,67.8 C 219.5,67 221,67.5 221.5,69" fill="none" stroke="#a5b4fc" stroke-width="0.75" opacity="0.55" stroke-linecap="round" />
        </g>
      </g>

      <!-- ==================== 10. INDICADOR LED DE ESTADO ==================== -->
      <g id="svg-led">
        <circle cx="173" cy="95" r="3.2" fill="#090d16" stroke="#334155" stroke-width="0.8" />
        <circle cx="173" cy="95" r="1.8" fill={$uiState === 'Connected' ? '#10b981' : '#f59e0b'} filter="url(#interactive-glow)" />
        <circle cx="172.5" cy="94.5" r="0.6" fill="#e0f2fe" />
      </g>
    </svg>
  </div>

  <!-- Fila de botones para selección rápida directamente bajo el blueprint -->
  <div class="grid grid-cols-2 gap-2 w-full max-w-[360px] mt-2 flex-shrink-0">
    {#each BUTTON_DEFINITIONS as btn}
      {@const actName = getButtonActionName(btn.cid, $statusData.mapping)}
      {@const isCustom = actName !== 'Por defecto del sistema'}
      <button
        type="button"
        on:click={() => selectButton(btn.cid)}
        class="flex flex-col text-left px-3 py-1.5 rounded-xl transition-all duration-150 border text-xs
          {$selectedCid === btn.cid 
            ? 'bg-indigo-600/25 border-indigo-500/70 shadow-sm shadow-indigo-500/20 text-white ring-1 ring-indigo-500/40' 
            : 'bg-slate-800/40 border-slate-700/50 hover:bg-slate-800/70 text-slate-300'}"
      >
        <div class="flex items-center justify-between w-full">
          <span class="font-medium truncate">{btn.name}</span>
          {#if isCustom}
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-sm shadow-emerald-400/50 flex-shrink-0 ml-1"></span>
          {/if}
        </div>
        <span class="text-[11px] {isCustom ? 'text-emerald-400' : 'text-slate-400'} truncate mt-0.5">
          {actName}
        </span>
      </button>
    {/each}
  </div>
</div>
