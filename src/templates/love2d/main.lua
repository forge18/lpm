---@diagnostic disable: undefined-global
-- {{project_name}}
-- Version: {{project_version}}

function love.load()
    -- Initialize game
    print("{{project_name}} loaded!")
end

function love.update(dt)
    -- Update game logic
end

function love.draw()
    -- Draw game
    love.graphics.print("Hello from {{project_name}}!", 10, 10)
end

