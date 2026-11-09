#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AuraTranslate Comprehensive Guide Generator
Assembles all sections into auratranslate-guide.html strictly adhering to IBM Carbon Design System v11.
"""

import os
import sys

# Add scratch to python path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from part_header_nav import get_header_and_sidenav
from part_ch1_ch2 import get_chapters_1_and_2
from part_ch3_ch4_ch5 import get_chapters_3_4_5
from part_ch6_ch7_ch8 import get_chapters_6_7_8
from part_ch9_ch10_ch11 import get_chapters_9_10_11
from part_ch12_ch13_ch14 import get_chapters_12_13_14

OUTPUT_FILE = "/Users/hoangnam/LocalSites/addon/AuraTranslate/auratranslate-guide.html"

def get_css():
    return """<!DOCTYPE html>
<html lang="vi" data-theme="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>AuraTranslate — Sổ tay Hướng dẫn Toàn diện & Mô tả Chức năng v2.0</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,400;0,500;0,600;1,400&family=IBM+Plex+Sans:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400&family=IBM+Plex+Serif:ital,wght@0,400;0,600;1,400&display=swap" rel="stylesheet">
  <style>
    :root {
      --cds-background: #161616;
      --cds-background-hover: #262626;
      --cds-layer-01: #262626;
      --cds-layer-02: #393939;
      --cds-layer-03: #525252;
      --cds-text-primary: #f4f4f4;
      --cds-text-secondary: #c6c6c6;
      --cds-text-placeholder: #8d8d8d;
      --cds-text-helper: #a8a8a8;
      --cds-text-on-color: #ffffff;
      --cds-border-subtle-01: #393939;
      --cds-border-subtle-02: #525252;
      --cds-border-strong-01: #6f6f6f;
      --cds-link-primary: #78a9ff;
      --cds-button-primary: #0f62fe;
      --cds-button-primary-hover: #0353e9;
      --cds-button-secondary: #6f6f6f;
      --cds-button-secondary-hover: #525252;
      --cds-button-tertiary: #ffffff;
      --cds-support-error: #fa4d56;
      --cds-support-success: #42be65;
      --cds-support-warning: #f1c21b;
      --cds-support-info: #4589ff;
      --cds-support-purple: #a56eff;
      --cds-support-cyan: #1192e8;
      --cds-support-teal: #009d9a;
      --cds-support-magenta: #ee5385;
      --cds-field-01: #262626;
      --cds-focus: #0f62fe;
      --cds-shadow: 0 4px 16px rgba(0,0,0,0.4);
      --font-sans: 'IBM Plex Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      --font-mono: 'IBM Plex Mono', Menlo, Monaco, monospace;
      --font-serif: 'IBM Plex Serif', Georgia, serif;
    }

    [data-theme="light"] {
      --cds-background: #f4f4f4;
      --cds-background-hover: #e8e8e8;
      --cds-layer-01: #ffffff;
      --cds-layer-02: #f4f4f4;
      --cds-layer-03: #e0e0e0;
      --cds-text-primary: #161616;
      --cds-text-secondary: #525252;
      --cds-text-placeholder: #a8a8a8;
      --cds-text-helper: #6f6f6f;
      --cds-text-on-color: #ffffff;
      --cds-border-subtle-01: #e0e0e0;
      --cds-border-subtle-02: #c6c6c6;
      --cds-border-strong-01: #8d8d8d;
      --cds-link-primary: #0f62fe;
      --cds-button-primary: #0f62fe;
      --cds-button-primary-hover: #0353e9;
      --cds-button-secondary: #393939;
      --cds-button-secondary-hover: #4c4c4c;
      --cds-button-tertiary: #161616;
      --cds-support-error: #da1e28;
      --cds-support-success: #24a148;
      --cds-support-warning: #f1c21b;
      --cds-support-info: #0043ce;
      --cds-support-purple: #8a3ffc;
      --cds-support-cyan: #0072c3;
      --cds-support-teal: #007d79;
      --cds-support-magenta: #d12771;
      --cds-field-01: #ffffff;
      --cds-shadow: 0 4px 16px rgba(0,0,0,0.1);
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    html { scroll-behavior: smooth; }
    body {
      font-family: var(--font-sans);
      background-color: var(--cds-background);
      color: var(--cds-text-primary);
      line-height: 1.5;
      font-size: 14px;
      -webkit-font-smoothing: antialiased;
      transition: background-color 0.2s ease, color 0.2s ease;
    }

    /* Carbon Header */
    .cds--header {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      height: 48px;
      background-color: var(--cds-background);
      border-bottom: 1px solid var(--cds-border-subtle-01);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 16px;
      z-index: 1000;
    }
    .cds--header__name {
      display: flex;
      align-items: center;
      gap: 8px;
      text-decoration: none;
      color: var(--cds-text-primary);
      font-size: 14px;
      font-weight: 600;
      letter-spacing: 0.1px;
    }
    .cds--header__badge {
      font-family: var(--font-mono);
      font-size: 11px;
      font-weight: 500;
      padding: 2px 8px;
      background-color: var(--cds-layer-02);
      border: 1px solid var(--cds-border-subtle-02);
      border-radius: 2px;
      color: var(--cds-support-cyan);
    }
    .cds--header__search {
      position: relative;
      width: 420px;
      max-width: 100%;
    }
    .cds--header__search input {
      width: 100%;
      height: 32px;
      background-color: var(--cds-field-01);
      border: 1px solid var(--cds-border-subtle-01);
      border-bottom: 1px solid var(--cds-border-strong-01);
      color: var(--cds-text-primary);
      font-family: var(--font-sans);
      font-size: 13px;
      padding: 0 32px 0 12px;
      outline: none;
      transition: border-color 0.15s ease;
    }
    .cds--header__search input:focus {
      border-color: var(--cds-focus);
      outline: 2px solid var(--cds-focus);
      outline-offset: -2px;
    }
    .cds--header__search-kbd {
      position: absolute;
      right: 8px;
      top: 50%;
      transform: translateY(-50%);
      font-family: var(--font-mono);
      font-size: 10.5px;
      color: var(--cds-text-placeholder);
      background: var(--cds-layer-02);
      padding: 1px 5px;
      border-radius: 2px;
      border: 1px solid var(--cds-border-subtle-02);
      pointer-events: none;
    }
    .cds--header__actions {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    /* Carbon SideNav */
    .cds--side-nav {
      position: fixed;
      top: 48px;
      left: 0;
      bottom: 0;
      width: 280px;
      background-color: var(--cds-layer-01);
      border-right: 1px solid var(--cds-border-subtle-01);
      overflow-y: auto;
      padding: 16px 0;
      z-index: 900;
    }
    .cds--side-nav__group-title {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: var(--cds-text-helper);
      padding: 12px 16px 6px;
    }
    .cds--side-nav__link {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 16px;
      color: var(--cds-text-secondary);
      text-decoration: none;
      font-size: 13px;
      border-left: 3px solid transparent;
      transition: background 0.15s ease, color 0.15s ease;
    }
    .cds--side-nav__link:hover {
      background-color: var(--cds-layer-02);
      color: var(--cds-text-primary);
    }
    .cds--side-nav__link.active {
      background-color: var(--cds-layer-02);
      color: var(--cds-text-primary);
      border-left-color: var(--cds-button-primary);
      font-weight: 600;
    }
    .cds--side-nav__count {
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--cds-text-placeholder);
      background: var(--cds-layer-03);
      padding: 1px 6px;
      border-radius: 10px;
    }

    /* Main Content Layout */
    .cds--main-content {
      margin-left: 280px;
      margin-top: 48px;
      padding: 40px 48px 100px;
      max-width: 1280px;
    }

    /* Typography & Hierarchy */
    h1.cds--page-title {
      font-size: 36px;
      font-weight: 300;
      line-height: 1.25;
      margin-bottom: 8px;
      letter-spacing: -0.5px;
    }
    .cds--page-subtitle {
      font-size: 16px;
      color: var(--cds-text-secondary);
      margin-bottom: 32px;
      max-width: 860px;
    }
    h2.cds--section-title {
      font-size: 24px;
      font-weight: 400;
      margin-top: 56px;
      margin-bottom: 16px;
      padding-bottom: 12px;
      border-bottom: 1px solid var(--cds-border-subtle-01);
      display: flex;
      align-items: center;
      gap: 12px;
    }
    h3.cds--subsection-title {
      font-size: 18px;
      font-weight: 600;
      margin-top: 28px;
      margin-bottom: 12px;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    p {
      color: var(--cds-text-secondary);
      margin-bottom: 16px;
      line-height: 1.6;
    }
    p strong {
      color: var(--cds-text-primary);
    }

    /* Carbon Components */
    .cds--tile {
      background-color: var(--cds-layer-01);
      border: 1px solid var(--cds-border-subtle-01);
      padding: 20px;
      margin-bottom: 20px;
      border-radius: 0;
      transition: border-color 0.15s ease, background-color 0.15s ease;
    }
    .cds--tile--clickable:hover {
      background-color: var(--cds-layer-02);
      border-color: var(--cds-border-strong-01);
      cursor: pointer;
    }
    .cds--grid-cards {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }
    .cds--card {
      background-color: var(--cds-layer-01);
      border: 1px solid var(--cds-border-subtle-01);
      padding: 20px;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
    }
    .cds--card-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 12px;
    }
    .cds--card-title {
      font-size: 16px;
      font-weight: 600;
      color: var(--cds-text-primary);
    }

    /* Carbon Tags */
    .cds--tag {
      display: inline-flex;
      align-items: center;
      font-size: 12px;
      font-weight: 500;
      line-height: 1;
      padding: 4px 8px;
      border-radius: 12px;
      background-color: var(--cds-layer-02);
      color: var(--cds-text-secondary);
      border: 1px solid transparent;
      gap: 4px;
    }
    .cds--tag--blue { background: rgba(15, 98, 254, 0.15); color: #78a9ff; border-color: rgba(15, 98, 254, 0.3); }
    .cds--tag--green { background: rgba(66, 190, 101, 0.15); color: #6fdc8c; border-color: rgba(66, 190, 101, 0.3); }
    .cds--tag--purple { background: rgba(165, 110, 255, 0.15); color: #d4bbff; border-color: rgba(165, 110, 255, 0.3); }
    .cds--tag--cyan { background: rgba(17, 146, 232, 0.15); color: #82cfff; border-color: rgba(17, 146, 232, 0.3); }
    .cds--tag--red { background: rgba(250, 77, 86, 0.15); color: #ff8389; border-color: rgba(250, 77, 86, 0.3); }
    .cds--tag--yellow { background: rgba(241, 194, 27, 0.15); color: #f1c21b; border-color: rgba(241, 194, 27, 0.3); }

    /* Carbon Callout / Alert */
    .cds--inline-notification {
      display: flex;
      gap: 12px;
      padding: 14px 16px;
      background-color: var(--cds-layer-01);
      border-left: 4px solid var(--cds-support-info);
      margin-bottom: 20px;
    }
    .cds--inline-notification--warning { border-left-color: var(--cds-support-warning); }
    .cds--inline-notification--success { border-left-color: var(--cds-support-success); }
    .cds--inline-notification--error { border-left-color: var(--cds-support-error); }
    .cds--inline-notification__content {
      font-size: 13.5px;
      color: var(--cds-text-secondary);
    }
    .cds--inline-notification__title {
      font-weight: 600;
      color: var(--cds-text-primary);
      margin-bottom: 2px;
    }

    /* Keyboard Shortcut Badges */
    kbd {
      font-family: var(--font-mono);
      font-size: 11.5px;
      padding: 2px 6px;
      background-color: var(--cds-layer-02);
      border: 1px solid var(--cds-border-strong-01);
      border-radius: 3px;
      box-shadow: 0 1px 1px rgba(0,0,0,0.2);
      color: var(--cds-text-primary);
      white-space: nowrap;
    }

    /* Carbon Tables */
    .cds--data-table-container {
      width: 100%;
      overflow-x: auto;
      margin-bottom: 24px;
      border: 1px solid var(--cds-border-subtle-01);
    }
    .cds--data-table {
      width: 100%;
      border-collapse: collapse;
      text-align: left;
      font-size: 13.5px;
    }
    .cds--data-table th {
      background-color: var(--cds-layer-02);
      color: var(--cds-text-primary);
      font-weight: 600;
      padding: 12px 16px;
      border-bottom: 1px solid var(--cds-border-subtle-01);
      white-space: nowrap;
    }
    .cds--data-table td {
      padding: 12px 16px;
      border-bottom: 1px solid var(--cds-border-subtle-01);
      color: var(--cds-text-secondary);
      background-color: var(--cds-layer-01);
      vertical-align: top;
    }
    .cds--data-table tr:hover td {
      background-color: var(--cds-layer-02);
      color: var(--cds-text-primary);
    }

    /* Buttons */
    .cds--btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      height: 36px;
      padding: 0 16px;
      font-family: var(--font-sans);
      font-size: 13px;
      font-weight: 500;
      text-decoration: none;
      cursor: pointer;
      border: none;
      background-color: var(--cds-button-primary);
      color: var(--cds-text-on-color);
      transition: background-color 0.15s ease;
    }
    .cds--btn:hover { background-color: var(--cds-button-primary-hover); }
    .cds--btn--secondary {
      background-color: var(--cds-button-secondary);
    }
    .cds--btn--secondary:hover { background-color: var(--cds-button-secondary-hover); }
    .cds--btn--tertiary {
      background-color: transparent;
      border: 1px solid var(--cds-border-strong-01);
      color: var(--cds-text-primary);
    }
    .cds--btn--tertiary:hover {
      background-color: var(--cds-layer-02);
    }
    .cds--btn--sm {
      height: 28px;
      padding: 0 10px;
      font-size: 11.5px;
    }
    .cds--btn--danger {
      background-color: var(--cds-support-error);
    }

    /* Interactive Simulators Styles */
    .aura-simulator {
      background-color: var(--cds-layer-01);
      border: 1px solid var(--cds-border-subtle-01);
      border-radius: 4px;
      overflow: hidden;
      margin: 24px 0 32px;
      box-shadow: var(--cds-shadow);
    }
    .aura-simulator-header {
      background-color: var(--cds-layer-02);
      border-bottom: 1px solid var(--cds-border-subtle-01);
      padding: 8px 16px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .aura-simulator-title {
      font-family: var(--font-mono);
      font-size: 12px;
      color: var(--cds-text-primary);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .aura-simulator-dots {
      display: flex;
      gap: 6px;
    }
    .aura-simulator-dot {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: var(--cds-border-strong-01);
    }
    .aura-simulator-dot.red { background: #ff5f56; }
    .aura-simulator-dot.yellow { background: #ffbd2e; }
    .aura-simulator-dot.green { background: #27c93f; }

    /* Workspace Dock Simulator */
    .aura-titlebar-mock {
      height: 38px;
      background: #1f1f1f;
      border-bottom: 1px solid var(--cds-border-subtle-01);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 12px;
    }
    .aura-modes-segmented {
      display: flex;
      background: #121212;
      border-radius: 4px;
      padding: 2px;
      gap: 2px;
    }
    .aura-mode-tab {
      padding: 3px 12px;
      font-size: 12px;
      color: var(--cds-text-secondary);
      cursor: pointer;
      border-radius: 3px;
      transition: all 0.15s ease;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .aura-mode-tab:hover { color: #fff; }
    .aura-mode-tab.active {
      background: var(--cds-button-primary);
      color: #fff;
      font-weight: 500;
    }
    .aura-workspace-body {
      display: grid;
      grid-template-columns: 1fr 380px;
      min-height: 480px;
      background: var(--cds-background);
    }
    .aura-workspace-body.layout-vertical {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr 300px;
    }
    .aura-grid-pane {
      border-right: 1px solid var(--cds-border-subtle-01);
      overflow-y: auto;
      background: var(--cds-background);
    }
    .aura-side-pane {
      display: flex;
      flex-direction: column;
      background: var(--cds-layer-01);
      overflow-y: auto;
    }
    .aura-side-pane-top {
      border-bottom: 1px solid var(--cds-border-subtle-01);
      flex: 1;
      padding: 12px;
    }
    .aura-side-pane-bottom {
      flex: 1;
      padding: 12px;
    }

    /* Grid Row Item */
    .aura-grid-row {
      border-bottom: 1px solid var(--cds-border-subtle-01);
      padding: 12px 16px;
      position: relative;
      transition: background 0.15s ease;
    }
    .aura-grid-row:hover { background: var(--cds-layer-01); }
    .aura-grid-row.active {
      background: var(--cds-layer-01);
      box-shadow: inset 3px 0 0 var(--cds-button-primary);
    }
    .aura-row-meta {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 6px;
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--cds-text-placeholder);
    }
    .aura-source-group {
      margin-bottom: 8px;
    }
    .aura-source-pinyin {
      font-family: var(--font-mono);
      font-size: 11.5px;
      color: var(--cds-support-cyan);
      letter-spacing: 0.3px;
      margin-bottom: 2px;
    }
    .aura-source-text {
      font-size: 15px;
      color: var(--cds-text-primary);
      font-weight: 500;
      margin-bottom: 2px;
      letter-spacing: 0.5px;
    }
    .aura-source-hanviet {
      font-size: 12px;
      color: var(--cds-support-purple);
      font-style: italic;
    }
    .aura-target-editor {
      width: 100%;
      background: var(--cds-field-01);
      border: 1px solid var(--cds-border-subtle-01);
      color: var(--cds-text-primary);
      font-family: var(--font-sans);
      font-size: 14px;
      line-height: 1.5;
      padding: 8px 10px;
      outline: none;
      border-radius: 2px;
      resize: vertical;
      min-height: 52px;
    }
    .aura-target-editor:focus {
      border-color: var(--cds-focus);
      outline: 2px solid var(--cds-focus);
    }

    /* Action Strips */
    .aura-strip {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 10px;
      background: var(--cds-layer-02);
      border-radius: 3px;
      margin-bottom: 8px;
      font-size: 12px;
    }
    .aura-strip.glossary { border-left: 3px solid var(--cds-support-purple); }
    .aura-strip.tm { border-left: 3px solid var(--cds-support-success); }
    .aura-strip.proofreader { border-left: 3px solid var(--cds-support-warning); }

    /* Context Menu Mock */
    .aura-context-menu {
      width: 220px;
      background: var(--cds-layer-02);
      border: 1px solid var(--cds-border-subtle-02);
      box-shadow: 0 8px 24px rgba(0,0,0,0.5);
      border-radius: 2px;
      padding: 4px 0;
      position: absolute;
      z-index: 100;
    }
    .aura-context-menu-item {
      padding: 6px 14px;
      font-size: 12.5px;
      color: var(--cds-text-secondary);
      display: flex;
      align-items: center;
      justify-content: space-between;
      cursor: pointer;
    }
    .aura-context-menu-item:hover {
      background: var(--cds-button-primary);
      color: #fff;
    }
    .aura-context-menu-divider {
      height: 1px;
      background: var(--cds-border-subtle-01);
      margin: 4px 0;
    }

    /* Modal Mock */
    .cds--modal {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,0.7);
      display: none;
      align-items: center;
      justify-content: center;
      z-index: 2000;
      padding: 20px;
    }
    .cds--modal.is-visible { display: flex; }
    .cds--modal-container {
      background: var(--cds-layer-01);
      border: 1px solid var(--cds-border-subtle-01);
      width: 680px;
      max-width: 100%;
      max-height: 85vh;
      display: flex;
      flex-direction: column;
      box-shadow: 0 12px 32px rgba(0,0,0,0.6);
    }
    .cds--modal-header {
      padding: 16px 20px;
      border-bottom: 1px solid var(--cds-border-subtle-01);
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .cds--modal-title {
      font-size: 16px;
      font-weight: 600;
    }
    .cds--modal-body {
      padding: 20px;
      overflow-y: auto;
      font-size: 13.5px;
    }
    .cds--modal-footer {
      padding: 12px 20px;
      background: var(--cds-layer-02);
      border-top: 1px solid var(--cds-border-subtle-01);
      display: flex;
      justify-content: flex-end;
      gap: 10px;
    }

    /* Step Process Component */
    .cds--steps {
      display: flex;
      flex-direction: column;
      gap: 16px;
      margin: 20px 0;
    }
    .cds--step-item {
      display: flex;
      gap: 16px;
      background: var(--cds-layer-01);
      border: 1px solid var(--cds-border-subtle-01);
      padding: 16px;
    }
    .cds--step-num {
      width: 32px;
      height: 32px;
      background: var(--cds-button-primary);
      color: #fff;
      font-weight: 600;
      font-size: 14px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }
    .cds--step-content h4 {
      font-size: 15px;
      font-weight: 600;
      color: var(--cds-text-primary);
      margin-bottom: 4px;
    }

    /* Diff Review Simulator */
    .diff-container {
      display: grid;
      grid-template-columns: 1fr 1fr;
      border: 1px solid var(--cds-border-subtle-01);
      background: var(--cds-layer-01);
      font-size: 13.5px;
    }
    .diff-col {
      padding: 16px;
    }
    .diff-col.original {
      border-right: 1px solid var(--cds-border-subtle-01);
      background: rgba(255,255,255,0.02);
    }
    .diff-del {
      background: rgba(250, 77, 86, 0.2);
      color: #ff8389;
      text-decoration: line-through;
      padding: 2px 4px;
    }
    .diff-add {
      background: rgba(66, 190, 101, 0.2);
      color: #6fdc8c;
      padding: 2px 4px;
      font-weight: 500;
    }

    /* Reading Mode Simulator */
    .reading-mode-preview {
      background: #1a1a1a;
      color: #e0e0e0;
      padding: 40px 60px;
      font-family: var(--font-serif);
      font-size: 17px;
      line-height: 1.8;
      max-width: 720px;
      margin: 0 auto;
      border: 1px solid var(--cds-border-subtle-01);
      position: relative;
    }
    .reading-frontier-line {
      border-bottom: 2px dashed var(--cds-support-cyan);
      margin: 20px 0;
      position: relative;
    }
    .reading-frontier-badge {
      position: absolute;
      right: 0;
      top: -12px;
      font-family: var(--font-mono);
      font-size: 11px;
      background: var(--cds-support-cyan);
      color: #000;
      padding: 2px 8px;
      font-weight: 600;
      border-radius: 2px;
    }

    @media (max-width: 1024px) {
      .cds--side-nav { display: none; }
      .cds--main-content { margin-left: 0; padding: 24px; }
      .aura-workspace-body { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
"""

def main():
    print("Generating comprehensive AuraTranslate guide...")
    content = []
    content.append(get_css())
    content.append(get_header_and_sidenav())
    content.append(get_chapters_1_and_2())
    content.append(get_chapters_3_4_5())
    content.append(get_chapters_6_7_8())
    content.append(get_chapters_9_10_11())
    content.append(get_chapters_12_13_14())

    full_html = "".join(content)
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write(full_html)
    
    line_count = len(full_html.splitlines())
    byte_count = len(full_html.encode("utf-8"))
    print(f"Generated successfully: {OUTPUT_FILE}")
    print(f"Total lines: {line_count}, Total bytes: {byte_count}")

if __name__ == "__main__":
    main()
