--!nonstrict

local M = {}

local BIN = noctalia.pluginDir() .. "/{{HELPER}}"
local DATA_DIR = noctalia.pluginDataDir() or noctalia.pluginDir()
local STATE_KEY = "{{PREFIX}}.state"

local ROOTS = {
  noctalia = noctalia,
  barWidget = barWidget,
  panel = panel,
  launcher = launcher,
  ui = ui,
}

local COLOR_ROLES = {
  "primary",
  "secondary",
  "tertiary",
  "error",
  "on_surface",
  "on_surface_variant",
  "outline_variant",
}

M.entry = "unknown"
M.config = {}
M.state = {}
M.host = {}
M.last_query = nil

local function materialize(value)
  if type(value) ~= "table" then
    return value
  end
  if type(value.__call) == "string" then
    local name = value.__call
    return function(...)
      M.call("action", { name = name, args = { ... } })
    end
  end
  if type(value.__host_callback) == "string" then
    local name = value.__host_callback
    return function(...)
      M.call("host_callback", { id = name, args = { ... } })
    end
  end
  local out = {}
  for key, item in pairs(value) do
    out[key] = materialize(item)
  end
  return out
end

local function render_node(node)
  local ctor = ui[node.kind]
  if not ctor then
    return ui.label({ text = "unknown ui node: " .. tostring(node.kind) })
  end
  local props = {}
  for key, item in pairs(node.props or {}) do
    props[key] = materialize(item)
  end
  local children = {}
  for index, child in ipairs(node.children or {}) do
    children[index] = render_node(child)
  end
  if #children == 0 then
    return ctor(props)
  end
  return ctor(props, children)
end

local function snapshot()
  local host = {
    now_ms = noctalia.nowMs(),
    dark = noctalia.isDarkMode(),
    outputs = noctalia.outputs(),
    clipboard = noctalia.clipboardText(),
    calendar = noctalia.calendarEvents(),
    media = noctalia.mediaPlayer(),
    stats = noctalia.systemStats(),
    nightlight = noctalia.nightlight.state(),
    compositor = noctalia.compositor.name(),
    colors = {},
  }
  for _, role in ipairs(COLOR_ROLES) do
    host.colors[role] = noctalia.getColor(role)
  end
  return host
end

local function resolve(root, path)
  local value = ROOTS[root]
  if not value then
    return nil
  end
  for _, part in ipairs(path) do
    if type(value) ~= "table" then
      return nil
    end
    value = value[part]
  end
  return value
end

local handlers = {
  log = function(op) noctalia.log(op.message) end,
  notify = function(op) noctalia.notify(op.title, op.body) end,
  notify_error = function(op) noctalia.notifyError(op.title, op.body) end,
  set_interval = function(op) noctalia.setUpdateInterval(op.ms) end,
  state_set = function(op) noctalia.state.set(op.key, op.value) end,
  toggle_panel = function(op) noctalia.togglePanel(op.id) end,
  open_settings = function(op) noctalia.openSettings() end,
  bar_text = function(op) barWidget.setText(op.text) end,
  bar_color = function(op) barWidget.setColor(op.role) end,
  bar_visible = function(op) barWidget.setVisible(op.visible) end,
  bar_tooltip = function(op) barWidget.setTooltip(op.value) end,
  bar_image = function(op)
    barWidget.setImage(op.path, op.preserve, op.width, op.height)
  end,
  bar_render = function(op) barWidget.render(render_node(op.tree)) end,
  panel_render = function(op) panel.render(render_node(op.tree)) end,
  desktop_widget_render = function(op)
    desktopWidget.render(render_node(op.tree))
  end,
  launcher_results = function(op) launcher.setResults(op.text, op.rows) end,
  layer_show = function(op) noctalia.layer.show(op.id) end,
  layer_hide = function(op) noctalia.layer.hide(op.id) end,
  layer_render = function(op) noctalia.layer.render(render_node(op.tree)) end,
  layer_configure = function(op) noctalia.layer.configure(op.id, op.props) end,
  remove_file = function(op) noctalia.removeFile(op.path) end,
  host_call = function(op)
    local target = resolve(op.root, op.path)
    if type(target) ~= "function" then
      noctalia.log("unknown host call: " .. tostring(op.root))
      return
    end
    local args = {}
    for index, value in ipairs(op.args) do
      args[index] = materialize(value)
    end
    local ok, err = pcall(target, table.unpack(args))
    if not ok then
      noctalia.log(tostring(err))
    end
  end,
}

function M.call(event, payload)
  M.host = snapshot()
  noctalia.runAsync({
    BIN,
    M.entry,
    event,
    DATA_DIR,
    noctalia.json.encode(payload or {}),
    noctalia.json.encode(M.config),
    noctalia.json.encode(M.state),
    noctalia.json.encode(M.host),
  }, function(result)
    if result.exitCode ~= 0 then
      noctalia.log(result.stderr)
      return
    end
    for line in string.gmatch(result.stdout, "[^\r\n]+") do
      local ok, op = pcall(noctalia.json.decode, line)
      if ok and type(op) == "table" and type(op.op) == "string" then
        local handler = handlers[op.op]
        if handler then
          handler(op)
        end
      end
    end
  end)
end

function M.setup(entry, configKeys)
  M.entry = entry
  M.config = {}
  for _, key in ipairs(configKeys) do
    M.config[key] = noctalia.getConfig(key)
  end
  M.state = noctalia.state.get(STATE_KEY) or {}
  M.host = snapshot()
end

function M.watch_state(rerender)
  noctalia.state.watch(STATE_KEY, function(value)
    M.state = value or {}
    rerender()
  end)
end

function M.query(text)
  M.last_query = text
  M.call("query", { text = text })
end

function M.rerender_query()
  if M.last_query ~= nil then
    M.call("query", { text = M.last_query })
  end
end

return M
