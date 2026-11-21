--!nonstrict

local M = {}

local BIN = noctalia.pluginDir() .. "/{{HELPER}}"
local DATA_DIR = noctalia.pluginDataDir() or noctalia.pluginDir()
local STATE_KEY = "{{PREFIX}}.state"

M.entry = "unknown"
M.config = {}
M.state = {}
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

local handlers = {
  log = function(op) noctalia.log(op.message) end,
  notify = function(op) noctalia.notify(op.title, op.body) end,
  set_interval = function(op) noctalia.setUpdateInterval(op.ms) end,
  state_set = function(op) noctalia.state.set(op.key, op.value) end,
  toggle_panel = function(op) noctalia.togglePanel(op.id) end,
  bar_text = function(op) barWidget.setText(op.text) end,
  bar_color = function(op) barWidget.setColor(op.role) end,
  bar_visible = function(op) barWidget.setVisible(op.visible) end,
  bar_tooltip = function(op) barWidget.setTooltip(op.value) end,
  bar_image = function(op)
    barWidget.setImage(op.path, op.preserve, op.width, op.height)
  end,
  bar_render = function(op) barWidget.render(render_node(op.tree)) end,
  panel_render = function(op) panel.render(render_node(op.tree)) end,
  launcher_results = function(op) launcher.setResults(op.text, op.rows) end,
  remove_file = function(op) noctalia.removeFile(op.path) end,
}

function M.call(event, payload)
  noctalia.runAsync({
    BIN,
    M.entry,
    event,
    DATA_DIR,
    noctalia.json.encode(payload or {}),
    noctalia.json.encode(M.config),
    noctalia.json.encode(M.state),
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
