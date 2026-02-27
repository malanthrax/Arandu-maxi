import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, Circle, FancyArrowPatch, Rectangle, Polygon, Wedge, Arc
import numpy as np
from matplotlib.font_manager import FontProperties
import matplotlib.patheffects as path_effects
from matplotlib.lines import Line2D

# Set up the canvas - wide landscape format for detail
fig, ax = plt.subplots(figsize=(20, 12), dpi=200)
ax.set_xlim(0, 200)
ax.set_ylim(0, 120)
ax.axis('off')
fig.patch.set_facecolor('#0a1628')

# Color palette
color_storage = '#0f2b3d'
color_transport = '#2d8cb3'
color_processing = '#d4a85c'
color_error = '#e07a5f'
color_data = '#f8f9fa'
color_metadata = '#6b7b8c'
color_background = '#0a1628'
color_highlight = '#4a9bc4'

# Fonts
font_title = FontProperties(family='sans-serif', size=14, weight='bold')
font_section = FontProperties(family='sans-serif', size=10, weight='bold')
font_label = FontProperties(family='sans-serif', size=7)
font_small = FontProperties(family='sans-serif', size=5)
font_mono = FontProperties(family='monospace', size=6)

# Background grid
for i in range(0, 201, 10):
    ax.axvline(x=i, color='#1a3a52', alpha=0.1, linewidth=0.3)
for i in range(0, 121, 10):
    ax.axhline(y=i, color='#1a3a52', alpha=0.1, linewidth=0.3)

# ============================================
# LAYER 1: STORAGE (Bottom) - GGUF Models
# ============================================
storage_y = 5
storage_height = 20

# Storage container
storage_box = FancyBboxPatch((5, storage_y), 55, storage_height, 
                             boxstyle="round,pad=0.02",
                             facecolor=color_storage, edgecolor=color_transport, 
                             linewidth=2, alpha=0.95)
ax.add_patch(storage_box)

ax.text(32.5, storage_y + storage_height - 2, 'LOCAL MODEL STORAGE', 
        ha='center', va='center', fontproperties=font_section, 
        color=color_transport, alpha=0.9)

# Multiple model files with details
models = [
    ('llama-3.1-8b-Q4_K_M.gguf', '8.5 GB'),
    ('mistral-7b-v0.3-Q5_K_M.gguf', '7.2 GB'),
    ('phi-3-mini-Q4_0.gguf', '2.1 GB'),
]

for i, (name, size) in enumerate(models):
    y_pos = storage_y + 13 - i * 5
    # File icon
    file_rect = FancyBboxPatch((10, y_pos), 45, 4, boxstyle="round,pad=0.01",
                               facecolor='#1a3f54', edgecolor=color_transport, 
                               linewidth=1, alpha=0.9)
    ax.add_patch(file_rect)
    # Fold
    fold_x = [55, 55, 50]
    fold_y = [y_pos + 4, y_pos + 1, y_pos + 4]
    ax.fill(fold_x, fold_y, color='#0d2535', alpha=0.9)
    ax.plot(fold_x, fold_y, color=color_transport, linewidth=0.8)
    # Filename
    ax.text(12, y_pos + 2, name, ha='left', va='center',
            fontproperties=font_mono, color=color_data, alpha=0.85)
    # Size
    ax.text(53, y_pos + 2, size, ha='right', va='center',
            fontsize=5, color=color_metadata, alpha=0.7)

# Storage metrics
ax.text(32.5, storage_y + 2, '3 models • 17.8 GB total', 
        ha='center', va='center', fontsize=6, color=color_metadata, alpha=0.8)

# ============================================
# LAYER 2: ARANDU DESKTOP (Center-Left)
# ============================================
arandu_x = 70
arandu_y = 35
arandu_w = 50
arandu_h = 50

# Main container
arandu_box = FancyBboxPatch((arandu_x, arandu_y), arandu_w, arandu_h,
                            boxstyle="round,pad=0.03",
                            facecolor='#132639', edgecolor=color_processing, 
                            linewidth=3, alpha=0.98)
ax.add_patch(arandu_box)

# Inner glow
inner_box = FancyBboxPatch((arandu_x+2, arandu_y+2), arandu_w-4, arandu_h-4,
                           boxstyle="round,pad=0.02",
                           facecolor='none', edgecolor=color_processing, 
                           linewidth=1, alpha=0.3)
ax.add_patch(inner_box)

ax.text(arandu_x + arandu_w/2, arandu_y + arandu_h - 4, 'ARANDU DESKTOP',
        ha='center', va='center', fontproperties=font_title, 
        color=color_processing, alpha=1.0)

# Desktop icons grid
for row in range(3):
    for col in range(4):
        x = arandu_x + 5 + col * 10
        y = arandu_y + 30 - row * 10
        icon = FancyBboxPatch((x, y), 7, 7, boxstyle="round,pad=0.01",
                              facecolor='#1a3f54', edgecolor=color_transport, 
                              linewidth=0.8, alpha=0.7)
        ax.add_patch(icon)

# Network widget detail
widget_box = FancyBboxPatch((arandu_x + 3, arandu_y + 5), 20, 12,
                            boxstyle="round,pad=0.01",
                            facecolor='#0d1f33', edgecolor=color_processing, 
                            linewidth=1.5, alpha=0.95)
ax.add_patch(widget_box)
ax.text(arandu_x + 13, arandu_y + 14, 'NETWORK SERVE', 
        ha='center', va='center', fontsize=6, color=color_processing, weight='bold')
ax.text(arandu_x + 13, arandu_y + 10, 'Status: ACTIVE', 
        ha='center', va='center', fontsize=5, color=color_transport)
ax.text(arandu_x + 13, arandu_y + 7, '10.0.0.47:8081', 
        ha='center', va='center', fontproperties=font_mono, 
        color=color_data, size=5)

# Status indicators
status_y = arandu_y + 22
for i, label in enumerate(['Proxy: ON', 'Server: RUNNING']):
    indicator = Circle((arandu_x + arandu_w - 8, status_y - i*4), 1.2, 
                       facecolor='#4caf50', alpha=0.9)
    ax.add_patch(indicator)
    ax.text(arandu_x + arandu_w - 5, status_y - i*4, label, 
            ha='left', va='center', fontsize=5, color=color_data, alpha=0.8)

# ============================================
# LAYER 3A: LLAMA.CPP SERVER (Center)
# ============================================
llama_x = 130
llama_y = 25
llama_w = 35
llama_h = 40

# Server container
llama_box = FancyBboxPatch((llama_x, llama_y), llama_w, llama_h,
                           boxstyle="round,pad=0.02",
                           facecolor='#0f2840', edgecolor=color_transport, 
                           linewidth=2.5, alpha=0.95)
ax.add_patch(llama_box)

ax.text(llama_x + llama_w/2, llama_y + llama_h - 3, 'LLAMA.CPP SERVER',
        ha='center', va='center', fontproperties=font_section, 
        color=color_transport, alpha=0.9)

# Server icon with rings
server_cx = llama_x + llama_w/2
server_cy = llama_y + 25
# Outer ring
outer_ring = Circle((server_cx, server_cy), 10, fill=False,
                    edgecolor=color_transport, linewidth=2, alpha=0.6)
ax.add_patch(outer_ring)
# Middle ring
mid_ring = Circle((server_cx, server_cy), 6, fill=False,
                  edgecolor=color_processing, linewidth=1.5, alpha=0.8)
ax.add_patch(mid_ring)
# Inner core
core = Circle((server_cx, server_cy), 3, facecolor=color_processing, alpha=0.95)
ax.add_patch(core)

# Port info
ax.text(server_cx, llama_y + 8, 'PORT 8080', 
        ha='center', va='center', fontproperties=font_mono,
        color=color_transport, size=7, weight='bold')
ax.text(server_cx, llama_y + 5, '127.0.0.1', 
        ha='center', va='center', fontsize=5, color=color_metadata)

# Current model indicator
model_indicator = FancyBboxPatch((llama_x + 3, llama_y + 12), llama_w - 6, 6,
                                 boxstyle="round,pad=0.01",
                                 facecolor='#0d1f33', edgecolor=color_processing, 
                                 linewidth=1, alpha=0.9)
ax.add_patch(model_indicator)
ax.text(server_cx, llama_y + 15, 'Active: llama-3.1-8b-Q4_K_M', 
        ha='center', va='center', fontsize=5, color=color_data, alpha=0.9)

# ============================================
# LAYER 3B: OPENAI PROXY (Center-Right)
# ============================================
proxy_x = 170
proxy_y = 25
proxy_w = 28
proxy_h = 40

# Proxy container
proxy_box = FancyBboxPatch((proxy_x, proxy_y), proxy_w, proxy_h,
                           boxstyle="round,pad=0.02",
                           facecolor='#0f2840', edgecolor=color_processing, 
                           linewidth=2.5, alpha=0.95)
ax.add_patch(proxy_box)

ax.text(proxy_x + proxy_w/2, proxy_y + proxy_h - 3, 'OPENAI PROXY',
        ha='center', va='center', fontproperties=font_section, 
        color=color_processing, alpha=0.9)

# Hexagon icon
hex_cx = proxy_x + proxy_w/2
hex_cy = proxy_y + 25
hex_r = 8
hex_angles = np.linspace(0, 2*np.pi, 7)[:-1] + np.pi/6
hex_x = hex_cx + hex_r * np.cos(hex_angles)
hex_y = hex_cy + hex_r * np.sin(hex_angles)
hexagon = plt.Polygon(list(zip(hex_x, hex_y)), facecolor='#0d1f33', 
                      edgecolor=color_processing, linewidth=2, alpha=0.95)
ax.add_patch(hexagon)
# Inner hex
hex_r2 = 4
hex_x2 = hex_cx + hex_r2 * np.cos(hex_angles)
hex_y2 = hex_cy + hex_r2 * np.sin(hex_angles)
hexagon2 = plt.Polygon(list(zip(hex_x2, hex_y2)), fill=False, 
                       edgecolor=color_processing, linewidth=1.5, alpha=0.6)
ax.add_patch(hexagon2)

# Port info
ax.text(hex_cx, proxy_y + 8, 'PORT 8081', 
        ha='center', va='center', fontproperties=font_mono,
        color=color_processing, size=7, weight='bold')
ax.text(hex_cx, proxy_y + 5, '0.0.0.0', 
        ha='center', va='center', fontsize=5, color=color_metadata)

# Endpoints list
endpoints = ['/v1/models', '/v1/chat/completions', '/health']
for i, endpoint in enumerate(endpoints):
    y = proxy_y + 14 - i * 3
    ax.text(proxy_x + 2, y, endpoint, ha='left', va='center',
            fontproperties=font_mono, size=4, color=color_data, alpha=0.7)

# ============================================
# LAYER 4: EXTERNAL CLIENTS (Right Side)
# ============================================
clients_x = 130
clients_y = 75
clients_w = 68
clients_h = 40

# Clients container
clients_box = FancyBboxPatch((clients_x, clients_y), clients_w, clients_h,
                             boxstyle="round,pad=0.02",
                             facecolor='#0d1f33', edgecolor=color_data, 
                             linewidth=1.5, alpha=0.8)
ax.add_patch(clients_box)

ax.text(clients_x + clients_w/2, clients_y + clients_h - 3, 'EXTERNAL CLIENTS',
        ha='center', va='center', fontproperties=font_section, 
        color=color_data, alpha=0.9)

# Client 1: Witsy
witsy_box = FancyBboxPatch((clients_x + 5, clients_y + 5), 18, 22,
                           boxstyle="round,pad=0.02",
                           facecolor='#1a3f54', edgecolor=color_data, 
                           linewidth=1, alpha=0.9)
ax.add_patch(witsy_box)
# Logo circle
witsy_logo = Circle((clients_x + 14, clients_y + 20), 5, 
                    facecolor='#2d5a7b', edgecolor=color_data, linewidth=1.5)
ax.add_patch(witsy_logo)
ax.text(clients_x + 14, clients_y + 20, 'W', ha='center', va='center',
        fontsize=12, color=color_data, weight='bold')
ax.text(clients_x + 14, clients_y + 12, 'Witsy', ha='center', va='center',
        fontproperties=font_label, color=color_data, size=6)
ax.text(clients_x + 14, clients_y + 8, 'Port: 8091', ha='center', va='center',
        fontproperties=font_mono, size=5, color=color_metadata)
ax.text(clients_x + 14, clients_y + 5, '/v1/', ha='center', va='center',
        fontproperties=font_mono, size=4, color=color_transport)

# Client 2: Cherry AI
cherry_box = FancyBboxPatch((clients_x + 28, clients_y + 5), 18, 22,
                            boxstyle="round,pad=0.02",
                            facecolor='#1a3f54', edgecolor=color_data, 
                            linewidth=1, alpha=0.9)
ax.add_patch(cherry_box)
cherry_logo = Circle((clients_x + 37, clients_y + 20), 5, 
                     facecolor='#2d5a7b', edgecolor=color_data, linewidth=1.5)
ax.add_patch(cherry_logo)
ax.text(clients_x + 37, clients_y + 20, 'C', ha='center', va='center',
        fontsize=12, color=color_data, weight='bold')
ax.text(clients_x + 37, clients_y + 12, 'Cherry AI', ha='center', va='center',
        fontproperties=font_label, color=color_data, size=6)
ax.text(clients_x + 37, clients_y + 8, 'Port: 8091', ha='center', va='center',
        fontproperties=font_mono, size=5, color=color_metadata)
ax.text(clients_x + 37, clients_y + 5, 'No /v1', ha='center', va='center',
        fontproperties=font_mono, size=4, color=color_transport)

# Client 3: Generic/Other
generic_box = FancyBboxPatch((clients_x + 51, clients_y + 5), 12, 22,
                             boxstyle="round,pad=0.02",
                             facecolor='#1a3f54', edgecolor=color_data, 
                             linewidth=1, alpha=0.7)
ax.add_patch(generic_box)
ax.text(clients_x + 57, clients_y + 20, '+', ha='center', va='center',
        fontsize=16, color=color_data, weight='bold')
ax.text(clients_x + 57, clients_y + 12, 'More', ha='center', va='center',
        fontsize=6, color=color_data, alpha=0.8)
ax.text(clients_x + 57, clients_y + 8, 'Clients', ha='center', va='center',
        fontsize=5, color=color_metadata, alpha=0.7)

# ============================================
# CONNECTION ARROWS & DATA FLOW
# ============================================

# Arrow style definitions
def draw_arrow(ax, start, end, color, label=None, label_pos=0.5, style='simple'):
    if style == 'simple':
        ax.annotate('', xy=end, xytext=start,
                   arrowprops=dict(arrowstyle='->', color=color, lw=1.5, 
                                  connectionstyle="arc3,rad=0.1"))
    elif style == 'thick':
        ax.annotate('', xy=end, xytext=start,
                   arrowprops=dict(arrowstyle='->', color=color, lw=2.5,
                                  connectionstyle="arc3,rad=0"))
    elif style == 'dashed':
        ax.annotate('', xy=end, xytext=start,
                   arrowprops=dict(arrowstyle='->', color=color, lw=1,
                                  connectionstyle="arc3,rad=0.1",
                                  linestyle='dashed', alpha=0.6))
    
    if label:
        mid_x = start[0] + (end[0] - start[0]) * label_pos
        mid_y = start[1] + (end[1] - start[1]) * label_pos
        ax.text(mid_x, mid_y + 1.5, label, ha='center', va='bottom',
                fontsize=5, color=color, alpha=0.8)

# Storage to Arandu
draw_arrow(ax, (60, 20), (70, 42), color_transport, 'Load Model')

# Arandu to llama.cpp
draw_arrow(ax, (120, 55), (130, 50), color_processing, 'Launch :8080')

# Arandu to Proxy
draw_arrow(ax, (120, 50), (170, 50), color_processing, 'Start :8081')

# Proxy to llama.cpp (bidirectional)
draw_arrow(ax, (170, 35), (165, 35), color_transport, 'HTTP', 0.5, 'thick')
draw_arrow(ax, (165, 33), (170, 33), color_transport, 'JSON', 0.5, 'thick')

# Proxy to Clients (fan out)
# Witsy connection
draw_arrow(ax, (170, 60), (145, 77), color_data, 'SSE/HTTP', 0.3)
# Cherry AI connection  
draw_arrow(ax, (170, 58), (165, 77), color_data, 'HTTP', 0.3)
# Generic connection
draw_arrow(ax, (170, 56), (180, 77), color_data, 'REST', 0.3)

# ============================================
# DATA PACKETS (Animated effect suggestion)
# ============================================
packet_positions = [
    (65, 30, color_data),    # Storage→Arandu
    (110, 52, color_processing),  # Arandu→llama
    (148, 52, color_processing),  # Arandu→Proxy
    (155, 34, color_transport),   # Proxy→llama
    (158, 68, color_data),   # Proxy→Clients
]

for x, y, color in packet_positions:
    packet = Circle((x, y), 1, facecolor=color, alpha=0.6)
    ax.add_patch(packet)

# ============================================
# LEGEND & INFO BOXES
# ============================================

# Protocol legend
legend_x = 5
legend_y = 85
legend_box = FancyBboxPatch((legend_x, legend_y), 40, 30,
                            boxstyle="round,pad=0.02",
                            facecolor='#0d1f33', edgecolor=color_metadata, 
                            linewidth=1, alpha=0.9)
ax.add_patch(legend_box)

ax.text(legend_x + 20, legend_y + 27, 'DATA FLOW', 
        ha='center', va='center', fontproperties=font_section, 
        color=color_data, size=8)

legend_items = [
    (color_transport, 'HTTP/TCP', 'Transport'),
    (color_processing, 'Processing', 'Server'),
    (color_data, 'OpenAI API', 'Protocol'),
    (color_error, 'Error/Warning', 'State'),
]

for i, (color, label, desc) in enumerate(legend_items):
    y = legend_y + 22 - i * 5
    indicator = Circle((legend_x + 3, y), 1.2, facecolor=color, alpha=0.9)
    ax.add_patch(indicator)
    ax.text(legend_x + 6, y, label, ha='left', va='center',
            fontsize=6, color=color_data, weight='bold')
    ax.text(legend_x + 25, y, desc, ha='left', va='center',
            fontsize=5, color=color_metadata)

# Technical details box
tech_x = 50
tech_y = 85
tech_box = FancyBboxPatch((tech_x, tech_y), 45, 30,
                          boxstyle="round,pad=0.02",
                          facecolor='#0d1f33', edgecolor=color_metadata, 
                          linewidth=1, alpha=0.9)
ax.add_patch(tech_box)

ax.text(tech_x + 22.5, tech_y + 27, 'TECHNICAL DETAILS', 
        ha='center', va='center', fontproperties=font_section, 
        color=color_data, size=8)

details = [
    '• Concurrent: Multiple clients OK',
    '• Sequential: One request at a time',
    '• Timeout: 5min request, 10s connect',
    '• Format: OpenAI-compatible JSON',
    '• Streaming: SSE for real-time',
]

for i, detail in enumerate(details):
    y = tech_y + 23 - i * 4
    ax.text(tech_x + 2, y, detail, ha='left', va='center',
            fontsize=5, color=color_data, alpha=0.85)

# ============================================
# TITLE & FOOTER
# ============================================

# Main title
ax.text(100, 117, 'ARANDU ARCHITECTURE', 
        ha='center', va='center', fontproperties=font_title, 
        color=color_data, size=18, weight='bold')
ax.text(100, 113, 'Local AI Infrastructure with OpenAI-Compatible API', 
        ha='center', va='center', fontsize=9, color=color_transport, alpha=0.9)

# Footer
footer_text = ('OpenAI-Compatible API  •  Concurrent Client Support  •  Sequential Model Processing  •  '
               'Port 8080 (llama.cpp)  •  Port 8081 (OpenAI Proxy)  •  Witsy/Cherry AI Compatible')
ax.text(100, 2, footer_text, 
        ha='center', va='center', fontsize=5, color=color_metadata, alpha=0.7)

# Corner decorations
corner_size = 4
# Top-left
ax.plot([2, 2+corner_size], [118, 118], color=color_processing, linewidth=2, alpha=0.6)
ax.plot([2, 2], [118-corner_size, 118], color=color_processing, linewidth=2, alpha=0.6)
# Top-right
ax.plot([198-corner_size, 198], [118, 118], color=color_processing, linewidth=2, alpha=0.6)
ax.plot([198, 198], [118-corner_size, 118], color=color_processing, linewidth=2, alpha=0.6)

plt.savefig('arandu-architecture-detailed.png', dpi=300, bbox_inches='tight', 
            facecolor=color_background, edgecolor='none', pad_inches=0.5)
