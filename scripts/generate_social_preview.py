import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

def create_social_preview():
    W, H = 1280, 640
    # 1. Canvas base con degradado suave oscuro moderno
    img = Image.new("RGBA", (W, H), (11, 15, 25, 255))
    draw = ImageDraw.Draw(img)

    # Ambient radial light en la esquina superior izquierda (halo azul/esmeralda elegante)
    glow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    for r in range(400, 0, -8):
        alpha = int(35 * (1.0 - r / 400.0))
        # Gradiente azul-índigo
        glow_draw.ellipse([80 - r, 80 - r, 80 + r, 80 + r], fill=(59, 130, 246, alpha))
        # Acento verde esmeralda (identidad MX)
        glow_draw.ellipse([260 - r, 200 - r, 260 + r, 200 + r], fill=(16, 185, 129, int(alpha * 0.7)))
    glow = glow.filter(ImageFilter.GaussianBlur(30))
    img = Image.alpha_composite(img, glow)
    draw = ImageDraw.Draw(img)

    # Borde sutil exterior estilo tarjeta
    draw.rounded_rectangle([16, 16, W - 16, H - 16], radius=24, outline=(30, 41, 59, 220), width=2)

    # 2. Tipografías
    font_title = ImageFont.truetype("/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf", 54)
    font_subtitle = ImageFont.truetype("/usr/share/fonts/truetype/ubuntu/Ubuntu-M.ttf", 20)
    font_sub_es = ImageFont.truetype("/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf", 17)
    font_pill = ImageFont.truetype("/usr/share/fonts/truetype/ubuntu/Ubuntu-M.ttf", 15)
    font_footer = ImageFont.truetype("/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf", 15)

    # 3. Icono de la aplicación
    icon_path = "crates/mx-gui/icons/512x512.png"
    try:
        app_icon = Image.open(icon_path).convert("RGBA")
        app_icon = app_icon.resize((96, 96), Image.Resampling.LANCZOS)
        
        # Máscara redondeada para el icono
        mask = Image.new("L", (96, 96), 0)
        mask_draw = ImageDraw.Draw(mask)
        mask_draw.rounded_rectangle([0, 0, 96, 96], radius=22, fill=255)
        
        # Sombra del icono
        icon_shadow = Image.new("RGBA", (112, 112), (0, 0, 0, 0))
        shadow_draw = ImageDraw.Draw(icon_shadow)
        shadow_draw.rounded_rectangle([6, 8, 102, 104], radius=22, fill=(0, 0, 0, 160))
        icon_shadow = icon_shadow.filter(ImageFilter.GaussianBlur(8))
        img.paste(icon_shadow, (68, 64), icon_shadow)
        
        img.paste(app_icon, (76, 72), mask)
    except Exception as e:
        print("Error cargando icono:", e)

    # 4. Textos de Cabecera (Izquierda)
    # Título
    draw.text((76, 192), "MX Pilot", font=font_title, fill=(248, 250, 252, 255))
    
    # Subtítulo bilingüe
    draw.text((76, 262), "Lightweight desktop Linux driver & GUI", font=font_subtitle, fill=(203, 213, 225, 255))
    draw.text((76, 290), "for Logitech MX Vertical & MX mice", font=font_subtitle, fill=(148, 163, 184, 255))
    draw.text((76, 326), "Suite nativa y liviana para ratones Logitech MX.", font=font_sub_es, fill=(100, 116, 139, 255))

    # 5. Badges / Pills
    pills = [
        ("Tauri v2 · Rust", (59, 130, 246)),
        ("4.17 MiB", (16, 185, 129)),
        ("Wayland & X11", (168, 85, 247)),
        ("Zero Root", (245, 158, 11)),
    ]
    px = 76
    py = 376
    for text, accent in pills:
        bbox = draw.textbbox((0, 0), text, font=font_pill)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]
        pw = tw + 24
        ph = 32
        
        # Fondo pill
        draw.rounded_rectangle([px, py, px + pw, py + ph], radius=16, fill=(15, 23, 42, 220), outline=(51, 65, 85, 200), width=1)
        # Punto de acento
        draw.ellipse([px + 10, py + 12, px + 18, py + 20], fill=accent)
        # Texto
        draw.text((px + 24, py + 7), text, font=font_pill, fill=(241, 245, 249, 255))
        px += pw + 12
        if px > 520:
            px = 76
            py += 40

    # 6. Footer Izquierdo
    draw.text((76, 566), "Linux  ·  Logitech MX  ·  GPL-3.0  ·  github.com/Panxio/mxpilot", font=font_footer, fill=(100, 116, 139, 255))

    # 7. Mockup de la Ventana de la App (Derecha)
    try:
        ss = Image.open("assets/screenshot.png").convert("RGBA")
        
        # Queremos que la ventana ocupe la derecha con altura de unos 480px
        target_h = 490
        target_w = int(ss.width * (target_h / ss.height))
        ss_resized = ss.resize((target_w, target_h), Image.Resampling.LANCZOS)
        
        # Coordenadas donde se ubicará (asomándose desde la derecha)
        wx = 560
        wy = 75
        
        # Sombra profunda tras la ventana
        win_shadow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        win_shadow_draw = ImageDraw.Draw(win_shadow)
        win_shadow_draw.rounded_rectangle([wx - 6, wy + 8, wx + target_w + 6, wy + target_h + 16], radius=18, fill=(0, 0, 0, 210))
        win_shadow = win_shadow.filter(ImageFilter.GaussianBlur(18))
        img = Image.alpha_composite(img, win_shadow)
        
        # Máscara redondeada para la ventana
        win_mask = Image.new("L", (target_w, target_h), 0)
        win_mask_draw = ImageDraw.Draw(win_mask)
        win_mask_draw.rounded_rectangle([0, 0, target_w, target_h], radius=14, fill=255)
        
        # Pegar captura en el lienzo
        img.paste(ss_resized, (wx, wy), win_mask)
        
        # Borde elegante alrededor de la ventana
        draw = ImageDraw.Draw(img)
        draw.rounded_rectangle([wx, wy, wx + target_w, wy + target_h], radius=14, outline=(71, 85, 105, 180), width=1)
    except Exception as e:
        print("Error integrando screenshot:", e)

    # 8. Guardar resultado final
    img.save("assets/social-preview.png", "PNG", optimize=True)
    print("Social preview generado exitosamente en assets/social-preview.png")

if __name__ == "__main__":
    create_social_preview()
