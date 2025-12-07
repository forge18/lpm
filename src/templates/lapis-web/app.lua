-- {{project_name}} - Lapis web application
-- Version: {{project_version}}

local lapis = require("lapis")
local app = lapis.Application()

app:get("/", function()
    return { render = "index" }
end)

app:get("/api/hello", function()
    return { json = { message = "Hello from {{project_name}}!" } }
end)

return app

