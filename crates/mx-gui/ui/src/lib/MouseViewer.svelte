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

<div class="relative flex flex-col items-center justify-center p-4 bg-slate-900/40 rounded-2xl border border-slate-800/80 shadow-inner backdrop-blur-sm">
  <!-- Badge de modelo y estado sobre el blueprint -->
  <div class="absolute top-3 left-4 flex items-center space-x-2 text-xs font-mono tracking-wider text-slate-400">
    <span class="w-2 h-2 rounded-full {$uiState === 'Connected' ? 'bg-emerald-500 shadow-sm shadow-emerald-500/50' : 'bg-amber-500'}"></span>
    <span>MX VERTICAL 57° BLUEPRINT</span>
  </div>

  {#if $uiState !== 'Connected'}
    <div class="absolute top-3 right-4 px-2 py-0.5 rounded text-[11px] font-medium bg-amber-500/10 border border-amber-500/30 text-amber-400">
      {$uiState === 'DaemonDown' ? 'Servicio Inactivo' : 'Dispositivo Desconectado'}
    </div>
  {/if}

  <!-- Contenedor del SVG Blueprint con escala responsiva -->
  <div class="w-full max-w-[420px] aspect-[360/320] my-2 transition-all duration-300 {$uiState !== 'Connected' ? 'opacity-40 grayscale-[40%]' : ''}">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 320" width="100%" height="100%" class="overflow-visible select-none">
      <defs>
        <!-- Gradiente para el chasis principal oscuro -->
        <linearGradient id="chassisGrad" x1="15%" y1="15%" x2="85%" y2="85%">
          <stop offset="0%" stop-color="#1e293b" />
          <stop offset="50%" stop-color="#1e293b" />
          <stop offset="100%" stop-color="#0f172a" />
        </linearGradient>

        <!-- Gradiente para la base y sombras de oclusión -->
        <linearGradient id="baseShadowGrad" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#090d16" />
          <stop offset="100%" stop-color="#05070a" />
        </linearGradient>

        <!-- Gradiente de la placa metálica de la cúspide (aluminio) -->
        <linearGradient id="metalPlateGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#cbd5e1" />
          <stop offset="30%" stop-color="#94a3b8" />
          <stop offset="70%" stop-color="#64748b" />
          <stop offset="100%" stop-color="#475569" />
        </linearGradient>

        <!-- Gradiente normal interactivo (indigo) -->
        <linearGradient id="interactiveGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#818cf8" />
          <stop offset="100%" stop-color="#4f46e5" />
        </linearGradient>

        <!-- Gradiente botón activo seleccionado (cyan vibrante) -->
        <linearGradient id="selectedGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#38bdf8" />
          <stop offset="100%" stop-color="#0284c7" />
        </linearGradient>

        <!-- Filtro de resplandor suave -->
        <filter id="softGlow" x="-30%" y="-30%" width="160%" height="160%">
          <feGaussianBlur stdDeviation="3.5" result="blur" />
          <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>

        <filter id="selectedGlow" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="5" result="blur" />
          <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>
      </defs>

      <!-- 0. Base oscura de apoyo y Oclusión -->
      <path d="M 50,278 
               C 50,278 120,290 280,285
               C 295,285 305,280 295,274
               C 285,268 250,268 210,268
               C 170,268 85,270 50,274
               Z" 
            fill="#05070a" opacity="0.65" />

      <!-- 1. Chasis Base (Grip inferior y borde inferior izquierdo) -->
      <path d="M 46,260 
               L 48,274 
               C 48,278 54,282 62,282 
               L 278,282 
               C 284,282 292,279 292,274 
               L 294,264 
               C 294,264 220,274 140,273
               C 80,272 46,260 46,260 Z" 
            fill="url(#baseShadowGrad)" stroke="#1e293b" stroke-width="1.5" />

      <!-- 2. Cuerpo Principal Ergonomía (Chasis de 57 Grados) -->
      <path id="chassis-main" 
            d="M 46,260 
               C 43,250 50,240 55,228
               C 70,195 105,135 125,103
               C 130,95 135,92 143,88
               L 218,52
               C 225,48 230,48 235,51
               L 242,56
               C 246,62 245,68 243,72
               C 235,90 238,105 244,120
               C 255,145 275,175 295,188
               C 310,198 322,204 322,212
               L 322,215
               C 322,222 300,245 294,264
               C 292,268 280,266 270,266
               C 210,265 110,264 46,260 Z" 
            fill="url(#chassisGrad)" stroke="#334155" stroke-width="1.2" />

      <!-- 3. Placa de Cúspide Metálica con Botón Superior (DPI / 0x00FD) -->
      <g id="top-plate">
        <path d="M 125,103 
                 C 127,97 132,92 140,88
                 L 218,52
                 C 225,48 229,48 234,51
                 L 242,56
                 C 245,61 243,65 240,68
                 L 142,106
                 C 134,109 127,108 125,103 Z" 
              fill="url(#metalPlateGrad)" stroke="#e2e8f0" stroke-width="0.8" />
        
        <!-- Bisel interior de la placa metálica -->
        <path d="M 128,100 C 132,95 138,91 142,89 L 216,55 C 221,52 225,52 228,54" 
              fill="none" stroke="#f1f5f9" stroke-width="0.7" opacity="0.9" />

        <!-- Botón de Conmutación DPI (id="svg-btn-top", CID 0x00FD) -->
        <g class="cursor-pointer group" on:click={() => selectButton(0x00FD)} on:keydown={(e) => handleKeyDown(e, 0x00FD)} role="button" tabindex="0" aria-label="Botón Superior DPI">
          <ellipse cx="218" cy="63" rx="16" ry="9" transform="rotate(-23, 218, 63)"
                   fill={$selectedCid === 0x00FD ? 'url(#selectedGrad)' : 'url(#interactiveGrad)'} 
                   stroke={$selectedCid === 0x00FD ? '#38bdf8' : '#818cf8'} 
                   stroke-width={$selectedCid === 0x00FD ? 2.4 : 1.2}
                   filter={$selectedCid === 0x00FD ? 'url(#selectedGlow)' : ''}
                   class="transition-all duration-200 group-hover:brightness-125" />
          <ellipse cx="218" cy="62" rx="12" ry="6" transform="rotate(-23, 218, 62)"
                   fill="#ffffff" opacity={$selectedCid === 0x00FD ? 0.35 : 0.15} pointer-events="none" />
        </g>
      </g>

      <!-- 4. Rueda de Scroll Frontal (CID 0x0052) -->
      <g class="cursor-pointer group" on:click={() => selectButton(0x0052)} on:keydown={(e) => handleKeyDown(e, 0x0052)} role="button" tabindex="0" aria-label="Rueda Central">
        <path d="M 103,132 
                 C 99,138 98,145 99,152 
                 C 101,153 103,153 104,151 
                 C 106,145 106,137 104,131 Z" 
              fill={$selectedCid === 0x0052 ? 'url(#selectedGrad)' : 'url(#interactiveGrad)'} 
              stroke={$selectedCid === 0x0052 ? '#38bdf8' : '#60a5fa'} 
              stroke-width={$selectedCid === 0x0052 ? 2.2 : 1}
              filter={$selectedCid === 0x0052 ? 'url(#selectedGlow)' : ''}
              class="transition-all duration-200 group-hover:brightness-125" />
        <!-- Muescas de la rueda -->
        <line x1="100.5" y1="135" x2="103.5" y2="136" stroke="#e0e7ff" stroke-width="0.8" />
        <line x1="99.2" y1="140" x2="102.5" y2="141" stroke="#e0e7ff" stroke-width="0.8" />
        <line x1="99" y1="145" x2="102.2" y2="146" stroke="#e0e7ff" stroke-width="0.8" />
        <line x1="99.5" y1="150" x2="102.5" y2="151" stroke="#e0e7ff" stroke-width="0.8" />
      </g>

      <!-- 5. Cortes de Piezas del Frontal (Estilo de Ingeniería) -->
      <path d="M 125,103 C 110,135 78,190 54,230" fill="none" stroke="#0f172a" stroke-width="1.8" opacity="0.9" />
      <path d="M 54,230 C 58,231 66,233 75,234 C 84,235 95,231 99,230" fill="none" stroke="#0f172a" stroke-width="1" opacity="0.6" />
      <path d="M 78,190 C 86,192 100,195 110,193 C 120,190 125,185 125,185" fill="none" stroke="#0f172a" stroke-width="1.2" opacity="0.75" />

      <!-- 6. Corte de la Cápsula Lateral del Pulgar -->
      <path d="M 142,106 
               C 147,125 158,135 174,136
               C 188,137 205,124 216,112
               C 224,103 234,88 240,68" 
            fill="none" stroke="#0f172a" stroke-width="1.5" opacity="0.8" />

      <!-- 7. Botones Laterales de Navegación -->
      <g>
        <!-- Botón Lateral Delantero (CID 0x0056) -->
        <g class="cursor-pointer group" on:click={() => selectButton(0x0056)} on:keydown={(e) => handleKeyDown(e, 0x0056)} role="button" tabindex="0" aria-label="Botón Lateral Adelante">
          <path d="M 178,124 
                   C 183,121 190,116 193,113 
                   C 195,111 194,109 191,110 
                   C 187,112 181,117 176,120 
                   C 174,122 175,125 178,124 Z" 
                fill={$selectedCid === 0x0056 ? 'url(#selectedGrad)' : 'url(#interactiveGrad)'} 
                stroke={$selectedCid === 0x0056 ? '#38bdf8' : '#60a5fa'} 
                stroke-width={$selectedCid === 0x0056 ? 2.2 : 1}
                filter={$selectedCid === 0x0056 ? 'url(#selectedGlow)' : ''}
                class="transition-all duration-200 group-hover:brightness-125" />
        </g>
        
        <!-- Botón Lateral Trasero (CID 0x0053) -->
        <g class="cursor-pointer group" on:click={() => selectButton(0x0053)} on:keydown={(e) => handleKeyDown(e, 0x0053)} role="button" tabindex="0" aria-label="Botón Lateral Atrás">
          <path d="M 194,112 
                   C 198,109 204,104 207,101 
                   C 209,99 208,97 205,98 
                   C 201,100 195,105 192,108 
                   C 190,110 191,113 194,112 Z" 
                fill={$selectedCid === 0x0053 ? 'url(#selectedGrad)' : 'url(#interactiveGrad)'} 
                stroke={$selectedCid === 0x0053 ? '#38bdf8' : '#60a5fa'} 
                stroke-width={$selectedCid === 0x0053 ? 2.2 : 1}
                filter={$selectedCid === 0x0053 ? 'url(#selectedGlow)' : ''}
                class="transition-all duration-200 group-hover:brightness-125" />
        </g>
      </g>

      <!-- 8. Indicador LED de Estado (id="svg-led") -->
      <circle cx="166" cy="131" r="2.5" fill={$uiState === 'Connected' ? '#10b981' : '#f59e0b'} filter="url(#softGlow)" />
      <circle cx="166" cy="131" r="0.8" fill="#ffffff" />

      <!-- 9. Área del Pulgar Texturizada / Estrías de Agarre -->
      <g opacity="0.25">
        <path d="M 195,142 C 202,150 208,172 201,215" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 203,140 C 213,148 221,170 213,218" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 211,138 C 224,146 234,168 225,221" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 220,134 C 236,144 246,166 238,223" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 228,130 C 248,141 259,165 251,226" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 237,126 C 260,138 272,163 264,228" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 246,122 C 272,135 284,161 278,231" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
        <path d="M 255,118 C 285,132 297,159 292,233" fill="none" stroke="#cbd5e1" stroke-width="1.5" />
      </g>

      <!-- 10. Puerto USB-C frontal -->
      <g>
        <path d="M 70,268 C 65,267 61,269 61,271 C 61,273 65,274 70,274 C 75,274 79,273 79,271 C 79,269 75,267 70,268 Z" 
              fill="#090d16" stroke="#334155" stroke-width="0.8" />
        <rect x="65" y="270" width="10" height="2" rx="0.5" fill="#1e293b" />
      </g>

      <!-- Resplandores de luz cenital -->
      <path d="M 128,105 C 105,135 80,185 55,225" fill="none" stroke="#ffffff" stroke-width="1.5" opacity="0.1" />
      <path d="M 145,108 C 158,118 180,126 195,115" fill="none" stroke="#ffffff" stroke-width="1" opacity="0.08" />
    </svg>
  </div>

  <!-- Fila de botones para selección rápida con tooltip/etiqueta -->
  <div class="grid grid-cols-2 gap-2 w-full mt-1">
    {#each BUTTON_DEFINITIONS as btn}
      <button
        type="button"
        on:click={() => selectButton(btn.cid)}
        class="flex flex-col text-left px-3 py-2 rounded-xl transition-all duration-150 border text-xs
          {$selectedCid === btn.cid 
            ? 'bg-indigo-600/20 border-indigo-500/60 shadow-sm shadow-indigo-500/20 text-white' 
            : 'bg-slate-800/40 border-slate-700/50 hover:bg-slate-800/70 text-slate-300'}"
      >
        <div class="flex items-center justify-between w-full">
          <span class="font-medium truncate">{btn.name}</span>
          <span class="text-[10px] font-mono text-slate-400">0x{btn.cid.toString(16).toUpperCase()}</span>
        </div>
        <span class="text-[11px] text-slate-400 truncate mt-0.5">
          {getButtonActionName(btn.cid, $statusData.mapping)}
        </span>
      </button>
    {/each}
  </div>
</div>
