mod audio;
mod build_pipeline;
mod chr;
mod converters;
mod emu;
mod emu_mcp;
mod famistudio;
mod ide_mcp;
mod map;
mod palettes;
mod project;
mod storage;
mod tracker;
mod watch;

use build_pipeline::BuildState;
use emu::EmuState;
use project::ProjectState;
use tauri::Manager;
use watch::WatchState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // Dev-only: lets AI agents (Claude Code) screenshot/inspect the live window
    // via a Unix socket at $TMPDIR/tauri-mcp.sock, bridged by `fc tauri-bridge`.
    #[cfg(debug_assertions)]
    {
        // start_socket_server defaults to false in PluginConfig::default(); use
        // PluginConfig::new(..) (sets it true) + a fixed socket path so the
        // `fc tauri-bridge` MCP server can always find it.
        let sock = std::path::PathBuf::from("/tmp/fc-tauri-mcp.sock");
        // The plugin panics with "address already in use" if a stale socket file
        // is left behind by a crashed/orphaned dev instance (the `tauri dev`
        // restart gotcha). Remove it before binding so restarts never collide.
        let _ = std::fs::remove_file(&sock);
        let cfg = tauri_plugin_mcp_gui::PluginConfig::new("fc-emulator".into())
            .socket_path(sock);
        builder = builder.plugin(tauri_plugin_mcp_gui::init_with_config(cfg));
    }
    builder
        .manage(EmuState::new())
        .manage(ProjectState::new())
        .manage(ide_mcp::IdeUiState::new())
        .manage(BuildState::new())
        .manage(WatchState::new())
        .setup(|app| {
            // Point the build pipeline at the bundled cc65 (ca65/ld65) so the
            // IDE "Build" works in installed apps, not just the dev tree.
            if let Ok(dir) = app.path().resource_dir() {
                build_pipeline::set_resource_dir(dir);
            }
            ide_mcp::start(app.handle().clone());
            emu_mcp::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            emu::open_rom,
            emu::open_rom_id,
            emu::poll_frame,
            emu::set_input,
            emu::set_speed,
            emu::set_volume,
            emu::set_remove_sprite_limit,
            emu::runtime_stats,
            emu_mcp::emu_mcp_status,
            ide_mcp::ide_ui_update,
            ide_mcp::ide_verify_game_ui,
            emu::list_palettes,
            emu::set_palette,
            emu::palette_preview,
            emu::load_palette_file,
            emu::screenshot,
            emu::export_state,
            emu::import_state,
            emu::control,
            emu::save_state,
            emu::load_state,
            emu::list_states,
            emu::delete_state,
            emu::list_library,
            emu::game_cover,
            emu::scan_library,
            emu::set_favorite,
            emu::remove_from_library,
            emu::remove_from_library_batch,
            emu::write_memory,
            emu::cpu_state,
            emu::ppu_apu_state,
            emu::event_dump,
            emu::set_event_breakpoint,
            emu::heatmap,
            emu::disassemble,
            emu::read_memory,
            emu::dbg_toggle_breakpoint,
            emu::dbg_add_breakpoint,
            emu::dbg_remove_breakpoint,
            emu::dbg_set_breakpoint_enabled,
            emu::dbg_breakpoints,
            emu::dbg_step,
            emu::dbg_resume,
            emu::add_cheat,
            emu::list_cheats,
            emu::set_cheat_enabled,
            emu::remove_cheat,
            emu::dbg_pattern,
            emu::dbg_nametable,
            emu::dbg_oam,
            emu::dbg_palette,
            project::project_new,
            project::project_open,
            project::project_get,
            project::project_save,
            project::project_file_tree,
            project::project_create_file,
            project::project_rename_file,
            project::project_delete_file,
            project::project_read_file,
            project::project_write_file,
            build_pipeline::build_run,
            build_pipeline::build_cancel,
            chr::chr_read,
            chr::chr_write,
            chr::chr_export_inc,
            map::map_read,
            map::map_write,
            converters::convert_png_to_chr,
            converters::convert_tiled_to_map,
            tracker::tracker_save,
            tracker::tracker_load,
            tracker::tracker_render,
            tracker::tracker_export,
            tracker::tracker_import_ftm,
            famistudio::famistudio_import,
            watch::watch_start,
            watch::watch_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
