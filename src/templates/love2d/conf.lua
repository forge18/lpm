---@diagnostic disable: undefined-global
-- Love2D configuration for {{project_name}}

function love.conf(t)
    t.title = "{{project_name}}"
    t.version = "11.5"
    t.console = true
    
    t.window.width = 800
    t.window.height = 600
    t.window.resizable = false
    t.window.vsync = true
end

