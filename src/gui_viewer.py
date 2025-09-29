#!/usr/bin/env python3

import os
import sys
import subprocess
from pathlib import Path
import tkinter as tk
from tkinter import ttk, messagebox
from PIL import Image, ImageTk
import threading

class WallpaperPicker:
    def __init__(self, root):
        self.root = root
        self.root.title("rswall - Wallpaper Theme Picker")
        self.root.geometry("1200x800")
        
        # Wallpaper directory
        self.wallpaper_dir = Path.home() / "Pictures" / "wallpapaer"
        self.wallpapers = []
        self.current_index = 0
        self.rswall_binary = Path.home() / ".config/rswall/target/release/rswall"
        
        # Create main frame
        main_frame = ttk.Frame(root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Configure grid weights
        root.columnconfigure(0, weight=1)
        root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(0, weight=1)
        main_frame.rowconfigure(1, weight=1)
        
        # Create toolbar
        toolbar = ttk.Frame(main_frame)
        toolbar.grid(row=0, column=0, sticky=(tk.W, tk.E), pady=(0, 10))
        
        # Navigation buttons
        self.prev_btn = ttk.Button(toolbar, text="◀ Previous", command=self.prev_wallpaper)
        self.prev_btn.pack(side=tk.LEFT, padx=5)
        
        self.next_btn = ttk.Button(toolbar, text="Next ▶", command=self.next_wallpaper)
        self.next_btn.pack(side=tk.LEFT, padx=5)
        
        # Apply button
        self.apply_btn = ttk.Button(toolbar, text="🎨 Apply Theme", command=self.apply_theme, style="Accent.TButton")
        self.apply_btn.pack(side=tk.LEFT, padx=20)
        
        # Refresh button
        self.refresh_btn = ttk.Button(toolbar, text="🔄 Refresh", command=self.load_wallpapers)
        self.refresh_btn.pack(side=tk.LEFT, padx=5)
        
        # Status label
        self.status_label = ttk.Label(toolbar, text="Ready")
        self.status_label.pack(side=tk.RIGHT, padx=10)
        
        # Image display area
        self.image_frame = ttk.LabelFrame(main_frame, text="Preview", padding="10")
        self.image_frame.grid(row=1, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        self.image_label = ttk.Label(self.image_frame)
        self.image_label.pack(expand=True, fill=tk.BOTH)
        
        # Info frame
        info_frame = ttk.Frame(main_frame)
        info_frame.grid(row=2, column=0, sticky=(tk.W, tk.E), pady=(10, 0))
        
        self.info_label = ttk.Label(info_frame, text="No image selected")
        self.info_label.pack(side=tk.LEFT)
        
        self.counter_label = ttk.Label(info_frame, text="0/0")
        self.counter_label.pack(side=tk.RIGHT)
        
        # Style configuration
        style = ttk.Style()
        style.configure("Accent.TButton", foreground="blue")
        
        # Bind keyboard shortcuts
        root.bind("<Left>", lambda e: self.prev_wallpaper())
        root.bind("<Right>", lambda e: self.next_wallpaper())
        root.bind("<Return>", lambda e: self.apply_theme())
        root.bind("<space>", lambda e: self.apply_theme())
        root.bind("r", lambda e: self.load_wallpapers())
        root.bind("q", lambda e: root.quit())
        
        # Load wallpapers
        self.load_wallpapers()
        
    def load_wallpapers(self):
        """Load all wallpapers from the directory"""
        self.status_label.config(text="Loading wallpapers...")
        self.wallpapers = []
        
        if self.wallpaper_dir.exists():
            for ext in ['*.jpg', '*.jpeg', '*.png', '*.webp', '*.gif', '*.bmp']:
                self.wallpapers.extend(self.wallpaper_dir.glob(ext))
        
        self.wallpapers.sort()
        
        if self.wallpapers:
            self.current_index = 0
            self.display_current_wallpaper()
            self.status_label.config(text=f"Loaded {len(self.wallpapers)} wallpapers")
        else:
            self.status_label.config(text="No wallpapers found")
            messagebox.showwarning("No Wallpapers", f"No wallpapers found in {self.wallpaper_dir}")
    
    def display_current_wallpaper(self):
        """Display the current wallpaper"""
        if not self.wallpapers:
            return
            
        wallpaper_path = self.wallpapers[self.current_index]
        
        try:
            # Load and resize image
            img = Image.open(wallpaper_path)
            
            # Calculate size to fit in window while maintaining aspect ratio
            display_width = 1000
            display_height = 600
            
            img_ratio = img.width / img.height
            display_ratio = display_width / display_height
            
            if img_ratio > display_ratio:
                # Image is wider
                new_width = display_width
                new_height = int(display_width / img_ratio)
            else:
                # Image is taller
                new_height = display_height
                new_width = int(display_height * img_ratio)
            
            img = img.resize((new_width, new_height), Image.Resampling.LANCZOS)
            
            # Convert to PhotoImage
            self.current_photo = ImageTk.PhotoImage(img)
            self.image_label.config(image=self.current_photo)
            
            # Update info
            self.info_label.config(text=f"{wallpaper_path.name} ({img.width}x{img.height})")
            self.counter_label.config(text=f"{self.current_index + 1}/{len(self.wallpapers)}")
            
        except Exception as e:
            self.status_label.config(text=f"Error loading image: {str(e)}")
    
    def prev_wallpaper(self):
        """Go to previous wallpaper"""
        if self.wallpapers and self.current_index > 0:
            self.current_index -= 1
            self.display_current_wallpaper()
    
    def next_wallpaper(self):
        """Go to next wallpaper"""
        if self.wallpapers and self.current_index < len(self.wallpapers) - 1:
            self.current_index += 1
            self.display_current_wallpaper()
    
    def apply_theme(self):
        """Apply the current wallpaper theme"""
        if not self.wallpapers:
            return
            
        wallpaper_path = self.wallpapers[self.current_index]
        self.status_label.config(text="Applying theme...")
        self.apply_btn.config(state="disabled")
        
        # Run rswall in a thread to avoid blocking GUI
        thread = threading.Thread(target=self._apply_theme_thread, args=(wallpaper_path,))
        thread.start()
    
    def _apply_theme_thread(self, wallpaper_path):
        """Apply theme in background thread"""
        try:
            # Run rswall
            result = subprocess.run(
                [str(self.rswall_binary), str(wallpaper_path), "--reload"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                # Set wallpaper using hyprpaper
                self._set_wallpaper(wallpaper_path)
                
                # Update UI in main thread
                self.root.after(0, lambda: self.status_label.config(text="✅ Theme applied successfully!"))
            else:
                error_msg = result.stderr or "Unknown error"
                self.root.after(0, lambda: self.status_label.config(text=f"❌ Error: {error_msg[:50]}..."))
                
        except Exception as e:
            self.root.after(0, lambda: self.status_label.config(text=f"❌ Error: {str(e)[:50]}..."))
        
        finally:
            self.root.after(0, lambda: self.apply_btn.config(state="normal"))
    
    def _set_wallpaper(self, wallpaper_path):
        """Set wallpaper using hyprpaper"""
        try:
            # Kill existing hyprpaper
            subprocess.run(["pkill", "-x", "hyprpaper"], capture_output=True)
            
            # Create hyprpaper config
            config_content = f"preload = {wallpaper_path}\nwallpaper = ,{wallpaper_path}\n"
            config_path = "/tmp/rswall_gui_hyprpaper.conf"
            
            with open(config_path, 'w') as f:
                f.write(config_content)
            
            # Start hyprpaper
            subprocess.Popen(["hyprpaper", "-c", config_path])
            
        except Exception as e:
            print(f"Error setting wallpaper: {e}")

def main():
    # Check if rswall binary exists
    rswall_bin = Path.home() / ".config/rswall/target/release/rswall"
    if not rswall_bin.exists():
        print(f"Error: rswall binary not found at {rswall_bin}")
        print("Please build rswall first with: cargo build --release")
        sys.exit(1)
    
    root = tk.Tk()
    app = WallpaperPicker(root)
    
    # Center window on screen
    root.update_idletasks()
    x = (root.winfo_screenwidth() // 2) - (1200 // 2)
    y = (root.winfo_screenheight() // 2) - (800 // 2)
    root.geometry(f"1200x800+{x}+{y}")
    
    root.mainloop()

if __name__ == "__main__":
    main()