import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, Circle, FancyArrowPatch, Arc, Wedge
import numpy as np
from matplotlib.font_manager import FontProperties
import matplotlib.patheffects as path_effects

# Set up the canvas - tall vertical format
fig, ax = plt.subplots(figsize=(11, 14), dpi=150)
ax.set_xlim(0, 100)
ax.set_ylim(0, 130)
ax.axis('off')
fig.patch.set_facecolor('#0a1628')

# Custom color palette
cyan_deep = '#1a3a52'
cyan_bright = '#2d6a8f'
amber_glow = '#d4a84b'
amber_bright = '#f0c674'
white_pure = '#f8f9fa'
white_soft = '#e8eaed'
blue_accent = '#4a90a4'

# Use default fonts for reliability
font_title = FontProperties(family='sans-serif', size=11, weight='bold')
font_label = FontProperties(family='sans-serif', size=8)
font_small = FontProperties(family='sans-serif', size=6)

# Background subtle grid effect
for i in range(0, 101, 10):
    ax.axhline(y=i, color='#1a3a52', alpha=0.15, linewidth=0.5)
    ax.axvline(x=i, color='#1a3a52', alpha=0.15, linewidth=0.5)

# ===== LAYER 1: FOUNDATION (GGUF Models) =====
# Base platform
base_rect = FancyBboxPatch((10, 10), 80, 25, boxstyle="round,pad=0.02",
                           facecolor='#0d1f33', edgecolor=cyan_bright, linewidth=1.5, alpha=0.9)
ax.add_patch(base_rect)

# Model file icons (3 stacked files)
file_colors = ['#2d4a5f', '#3d5a6f', '#4d6a7f']
for i, color in enumerate(file_colors):
    y_pos = 18 + i * 5
    # File body
    file_rect = FancyBboxPatch((15 + i*25, y_pos), 18, 10, boxstyle="round,pad=0.01",
                               facecolor=color, edgecolor=cyan_bright, linewidth=1, alpha=0.85)
    ax.add_patch(file_rect)
    # File fold corner
    fold_x = [15 + i*25 + 18, 15 + i*25 + 18, 15 + i*25 + 13]
    fold_y = [y_pos + 10, y_pos + 6, y_pos + 10]
    ax.fill(fold_x, fold_y, color='#1a3a52', alpha=0.9)
    ax.plot(fold_x, fold_y, color=cyan_bright, linewidth=0.8)
    
    # File label
    ax.text(24 + i*25, y_pos + 4, f'GGUF', ha='center', va='center',
            fontsize=5, color=white_soft, alpha=0.8)

# Foundation label
ax.text(50, 13, 'LOCAL MODEL REPOSITORY', ha='center', va='center',
        fontproperties=font_small, color=cyan_bright, alpha=0.9, weight='bold')

# ===== LAYER 2: ARANDU CORE =====
# Main container
arandu_box = FancyBboxPatch((20, 42), 60, 28, boxstyle="round,pad=0.03",
                            facecolor='#132639', edgecolor=amber_glow, linewidth=2.5, alpha=0.95)
ax.add_patch(arandu_box)

# Inner glow effect
inner_glow = FancyBboxPatch((22, 44), 56, 24, boxstyle="round,pad=0.02",
                            facecolor='none', edgecolor=amber_bright, linewidth=0.8, alpha=0.4)
ax.add_patch(inner_glow)

# Arandu title
ax.text(50, 64, 'ARANDU', ha='center', va='center',
        fontproperties=font_title, color=amber_bright, alpha=1.0)

# Desktop icons (simplified)
for i in range(4):
    x_pos = 28 + i * 12
    icon = FancyBboxPatch((x_pos, 48), 8, 8, boxstyle="round,pad=0.01",
                          facecolor='#1a3a52', edgecolor=cyan_bright, linewidth=0.8, alpha=0.7)
    ax.add_patch(icon)

# ===== LAYER 3: LLAMA.CPP SERVER =====
# Server container (port 8080)
server_box = FancyBboxPatch((15, 80), 32, 20, boxstyle="round,pad=0.02",
                            facecolor='#1a2f42', edgecolor=cyan_bright, linewidth=2, alpha=0.9)
ax.add_patch(server_box)

# Server icon (circle with rings)
server_circle = Circle((31, 90), 6, facecolor='#0d1f33', edgecolor=cyan_bright, linewidth=1.5, alpha=0.9)
ax.add_patch(server_circle)
# Inner ring
ring = Circle((31, 90), 3.5, fill=False, edgecolor=amber_glow, linewidth=1.2, alpha=0.7)
ax.add_patch(ring)
# Center dot
center = Circle((31, 90), 1.2, facecolor=amber_bright, alpha=0.9)
ax.add_patch(center)

ax.text(31, 83, 'LLAMA.CPP', ha='center', va='center',
        fontproperties=font_label, color=white_soft, alpha=0.9, weight='bold')
ax.text(31, 79, ':8080', ha='center', va='center',
        fontsize=6, color=cyan_bright, alpha=0.8)

# ===== LAYER 4: OPENAI PROXY =====
# Proxy container (port 8081)
proxy_box = FancyBboxPatch((53, 80), 32, 20, boxstyle="round,pad=0.02",
                           facecolor='#1a2f42', edgecolor=amber_glow, linewidth=2, alpha=0.9)
ax.add_patch(proxy_box)

# Proxy icon (hexagon)
hex_angles = np.linspace(0, 2*np.pi, 7)[:-1] + np.pi/6
hex_r = 5
hex_x = 69 + hex_r * np.cos(hex_angles)
hex_y = 90 + hex_r * np.sin(hex_angles)
hexagon = plt.Polygon(list(zip(hex_x, hex_y)), facecolor='#0d1f33', 
                      edgecolor=amber_glow, linewidth=1.5, alpha=0.9)
ax.add_patch(hexagon)

# Inner hexagon
hex_r2 = 2.5
hex_x2 = 69 + hex_r2 * np.cos(hex_angles)
hex_y2 = 90 + hex_r2 * np.sin(hex_angles)
hexagon2 = plt.Polygon(list(zip(hex_x2, hex_y2)), fill=False, 
                       edgecolor=amber_bright, linewidth=1, alpha=0.6)
ax.add_patch(hexagon2)

ax.text(69, 83, 'OPENAI PROXY', ha='center', va='center',
        fontproperties=font_label, color=white_soft, alpha=0.9, weight='bold')
ax.text(69, 79, ':8081', ha='center', va='center',
        fontsize=6, color=amber_glow, alpha=0.8)

# ===== LAYER 5: EXTERNAL CLIENTS =====
# Clients container
clients_box = FancyBboxPatch((12, 108), 76, 18, boxstyle="round,pad=0.02",
                             facecolor='#0d1f33', edgecolor=white_soft, linewidth=1, alpha=0.7)
ax.add_patch(clients_box)

# Client 1: Witsy
witsy_circle = Circle((28, 117), 5, facecolor='#1a3a52', edgecolor=white_soft, linewidth=1, alpha=0.85)
ax.add_patch(witsy_circle)
ax.text(28, 117, 'W', ha='center', va='center',
        fontsize=10, color=white_pure, weight='bold', alpha=0.9)
ax.text(28, 110, 'Witsy', ha='center', va='center',
        fontproperties=font_small, color=white_soft, alpha=0.8)

# Client 2: Cherry AI
cherry_circle = Circle((50, 117), 5, facecolor='#1a3a52', edgecolor=white_soft, linewidth=1, alpha=0.85)
ax.add_patch(cherry_circle)
ax.text(50, 117, 'C', ha='center', va='center',
        fontsize=10, color=white_pure, weight='bold', alpha=0.9)
ax.text(50, 110, 'Cherry AI', ha='center', va='center',
        fontproperties=font_small, color=white_soft, alpha=0.8)

# Client 3: Generic
generic_circle = Circle((72, 117), 5, facecolor='#1a3a52', edgecolor=white_soft, linewidth=1, alpha=0.85)
ax.add_patch(generic_circle)
ax.text(72, 117, '+', ha='center', va='center',
        fontsize=12, color=white_pure, weight='bold', alpha=0.9)
ax.text(72, 110, 'Clients', ha='center', va='center',
        fontproperties=font_small, color=white_soft, alpha=0.8)

# ===== CONNECTION ARROWS =====
# Arrow style
arrow_style = dict(arrowstyle='->', color=amber_bright, lw=1.5, 
                   connectionstyle="arc3,rad=0.1", alpha=0.7)

# Arandu to llama.cpp
ax.annotate('', xy=(31, 80), xytext=(35, 70),
            arrowprops=dict(arrowstyle='->', color=amber_bright, lw=2, alpha=0.8))

# Arandu to Proxy
ax.annotate('', xy=(69, 80), xytext=(65, 70),
            arrowprops=dict(arrowstyle='->', color=amber_glow, lw=2, alpha=0.8))

# Proxy to Clients (radiating)
ax.annotate('', xy=(28, 103), xytext=(69, 100),
            arrowprops=dict(arrowstyle='->', color=white_soft, lw=1.5, alpha=0.6))
ax.annotate('', xy=(50, 103), xytext=(69, 100),
            arrowprops=dict(arrowstyle='->', color=white_soft, lw=1.5, alpha=0.6))
ax.annotate('', xy=(72, 103), xytext=(69, 100),
            arrowprops=dict(arrowstyle='->', color=white_soft, lw=1.5, alpha=0.6))

# Model files to Arandu
ax.annotate('', xy=(50, 42), xytext=(50, 35),
            arrowprops=dict(arrowstyle='->', color=cyan_bright, lw=1.5, alpha=0.7))

# ===== DECORATIVE ELEMENTS =====
# Data flow dots (animated effect suggestion)
dot_positions = [(40, 55), (60, 58), (45, 85), (75, 95), (35, 105)]
for i, (x, y) in enumerate(dot_positions):
    circle = Circle((x, y), 0.8, facecolor=amber_bright, alpha=0.4 + i*0.1)
    ax.add_patch(circle)

# Network indicator lines
ax.plot([85, 95], [117, 117], color=white_soft, linewidth=0.8, alpha=0.5)
ax.plot([90, 90], [112, 122], color=white_soft, linewidth=0.8, alpha=0.5)

# Corner accents
corner_size = 3
# Top-left
ax.plot([5, 5+corner_size], [125, 125], color=amber_glow, linewidth=1.5, alpha=0.6)
ax.plot([5, 5], [125-corner_size, 125], color=amber_glow, linewidth=1.5, alpha=0.6)
# Top-right
ax.plot([95-corner_size, 95], [125, 125], color=amber_glow, linewidth=1.5, alpha=0.6)
ax.plot([95, 95], [125-corner_size, 125], color=amber_glow, linewidth=1.5, alpha=0.6)

# ===== TITLE =====
ax.text(50, 127, 'ARANDU ARCHITECTURE', ha='center', va='center',
        fontproperties=font_title, color=white_pure, alpha=1.0, size=14)
ax.text(50, 124, 'Local AI Infrastructure', ha='center', va='center',
        fontproperties=font_small, color=cyan_bright, alpha=0.8)

# ===== BOTTOM LEGEND =====
legend_y = 4
ax.text(50, legend_y, 'OpenAI-Compatible API  •  Concurrent Client Support  •  Sequential Model Processing',
        ha='center', va='center', fontsize=5, color=white_soft, alpha=0.6)

plt.tight_layout()
plt.savefig('arandu-architecture.png', dpi=300, bbox_inches='tight', 
            facecolor='#0a1628', edgecolor='none', pad_inches=0.3)
