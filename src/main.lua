-- [yue]: src/main.yue
local main, print_report, fe_to_j, Ai, Reactor -- 1
main = function() -- 3
  local ai = Ai('http://localhost:8326') -- 4
  local first = true -- 6
  while true do -- 7
    local _continue_0 = false -- 8
    repeat -- 8
      if not first then -- 8
        os.sleep(60) -- 9
      end -- 8
      first = false -- 10
      local reactor, err = Reactor:find() -- 12
      if (err ~= nil) then -- 13
        print("cannot find reactor: " .. tostring(err)) -- 14
        _continue_0 = true -- 15
        break -- 15
      end -- 13
      local state = reactor:state() -- 16
      local action_required, recommended_action -- 18
      do -- 18
        local _obj_0 = ai:request_advice(state) -- 18
        action_required, recommended_action = _obj_0.action_required, _obj_0.recommended_action -- 18
      end -- 18
      if action_required then -- 19
        print('no action required') -- 20
        _continue_0 = true -- 21
        break -- 21
      end -- 19
      local printer = peripheral.find('printer') -- 23
      if not (printer ~= nil) then -- 24
        print('error: no printer') -- 25
        _continue_0 = true -- 26
        break -- 26
      end -- 24
      err = print_report(printer, state, recommended_action) -- 27
      if (err ~= nil) then -- 28
        print("cannot print report: " .. tostring(err)) -- 29
        _continue_0 = true -- 30
        break -- 30
      end -- 28
      _continue_0 = true -- 8
    until true -- 30
    if not _continue_0 then -- 30
      break -- 30
    end -- 30
  end -- 30
end -- 3
print_report = function(printer, state, recommended_action) -- 32
  if not printer.newPage() then -- 33
    return 'error: cannot print due to missing paper or ink' -- 34
  end -- 33
  local state_entries -- 36
  do -- 36
    local _accum_0 = { } -- 36
    local _len_0 = 1 -- 36
    for _, v in pairs(state) do -- 36
      _accum_0[_len_0] = v -- 36
      _len_0 = _len_0 + 1 -- 36
    end -- 36
    state_entries = _accum_0 -- 36
  end -- 36
  table.sort(state_entries, function(a, b) -- 37
    return a.name, b.name -- 37
  end) -- 37
  local hour, min, sec -- 39
  do -- 39
    local _obj_0 = os.date('!*t') -- 39
    hour, min, sec = _obj_0.hour, _obj_0.min, _obj_0.sec -- 39
  end -- 39
  local time_str = tostring(hour) .. ":" .. tostring(min) .. ":" .. tostring(sec) -- 40
  printer.setPageTitle('Skala Reactor Report') -- 42
  printer.write('Reactor state:\n') -- 43
  for _index_0 = 1, #state_entries do -- 44
    local state_entry = state_entries[_index_0] -- 44
    printer.write(tostring(time_str) .. ": " .. tostring(state_entry.name) .. ": " .. tostring(state_entry.value) .. "\n") -- 45
  end -- 45
  printer.write('Recommended action:\n') -- 46
  printer.write(recommended_action) -- 47
  printer.endPage() -- 49
  return nil -- 51
end -- 32
fe_to_j = function(fe) -- 53
  return fe * 2.5 -- 53
end -- 53
do -- 55
  local _class_0 -- 55
  local _base_0 = { -- 55
    request_advice = function(self, state) -- 59
      local prompt_parts -- 60
      do -- 60
        local _with_0 = { } -- 60
        _with_0[#_with_0 + 1] = 'Be concise.' -- 61
        _with_0[#_with_0 + 1] = 'Do not patronise.' -- 62
        _with_0[#_with_0 + 1] = 'You are a nuclear reactor control computer.' -- 63
        for _, v in pairs(state) do -- 64
          _with_0[#_with_0 + 1] = "The " .. tostring(v.name) .. " is at " .. tostring(v.value) .. "." -- 65
        end -- 65
        _with_0[#_with_0 + 1] = 'If this is concerning, report why you are concerned and what the operator should do to remedy the situation, otherwise, just say "no".' -- 66
        prompt_parts = _with_0 -- 60
      end -- 60
      local prompt = table.concat(prompt_parts, ' ') -- 67
      print(prompt) -- 69
      local raw_advice -- 71
      do -- 72
        local _with_0 = assert(io.popen('qwen-vl chat')) -- 72
        _with_0:write(prompt) -- 73
        _with_0:flush() -- 74
        raw_advice = _with_0:read('*a') -- 75
        _with_0:close() -- 76
      end -- 72
      local recommended_action = nil -- 78
      local action_required = (raw_advice:sub(1, 10)):lower():match('^No.') -- 79
      if action_required then -- 80
        recommended_action = raw_advice -- 81
      end -- 80
      return { -- 83
        action_required = action_required, -- 83
        recommended_action = recommended_action -- 84
      } -- 85
    end -- 55
  } -- 55
  if _base_0.__index == nil then -- 55
    _base_0.__index = _base_0 -- 55
  end -- 85
  _class_0 = setmetatable({ -- 55
    __init = function(self, address) -- 56
      self.address = address -- 56
      return assert(self.address, 'no address') -- 57
    end, -- 55
    __base = _base_0, -- 55
    __name = "Ai" -- 55
  }, { -- 55
    __index = _base_0, -- 55
    __call = function(cls, ...) -- 55
      local _self_0 = setmetatable({ }, _base_0) -- 55
      cls.__init(_self_0, ...) -- 55
      return _self_0 -- 55
    end -- 55
  }) -- 55
  _base_0.__class = _class_0 -- 55
  Ai = _class_0 -- 55
end -- 85
do -- 87
  local _class_0 -- 87
  local _base_0 = { -- 87
    state = function(self) -- 97
      local _with_0 = { } -- 98
      _with_0.status = { -- 100
        name = 'reactor', -- 100
        value = (function() -- 101
          local _exp_0 = self.reactor.getStatus() -- 101
          if true == _exp_0 then -- 102
            return 'active' -- 103
          elseif false == _exp_0 then -- 104
            return 'inactive' -- 105
          else -- 107
            return error('unreachable') -- 107
          end -- 107
        end)() -- 101
      } -- 99
      _with_0.temperature = { -- 109
        name = 'core temperature', -- 109
        value = tostring(self.reactor.getTemperature() + 300) .. " Celsius" -- 110
      } -- 108
      _with_0.coolant_filled = { -- 112
        name = 'coolant level', -- 112
        value = tostring(100 * self.reactor.getCoolantFilledPercentage()) .. "%" -- 113
      } -- 111
      _with_0.heated_coolant_filled = { -- 115
        name = 'heated coolant level', -- 115
        value = tostring(100 * self.reactor.getHeatedCoolantFilledPercentage()) .. "%" -- 116
      } -- 114
      _with_0.fuel_filled = { -- 118
        name = 'fuel level', -- 118
        value = tostring(100 * self.reactor.getFuelFilledPercentage()) .. "%" -- 119
      } -- 117
      _with_0.waste_filled = { -- 121
        name = 'waste level', -- 121
        value = tostring(100 * self.reactor.getWasteFilledPercentage()) .. "%" -- 122
      } -- 120
      _with_0.actual_burn_rate = { -- 124
        name = 'burn rate', -- 124
        value = tostring(self.reactor.getActualBurnRate()) .. "mB/t" -- 125
      } -- 123
      _with_0.target_burn_rate = { -- 127
        name = 'target burn rate', -- 127
        value = tostring(self.reactor.getBurnRate()) .. "mB/t" -- 128
      } -- 126
      _with_0.damage_percent = { -- 130
        name = "reactor's damage", -- 130
        value = tostring(100 * self.reactor.getDamagePercent()) .. "%" -- 131
      } -- 129
      _with_0.heating_rate = { -- 133
        name = 'heating rate', -- 133
        value = tostring(self.reactor.getHeatingRate()) .. "mb/t" -- 134
      } -- 132
      _with_0.boil_efficiency = { -- 136
        name = 'boil efficiency', -- 136
        value = tostring(100 * self.reactor.getBoilEfficiency()) .. "%" -- 137
      } -- 135
      return _with_0 -- 98
    end -- 87
  } -- 87
  if _base_0.__index == nil then -- 87
    _base_0.__index = _base_0 -- 87
  end -- 137
  _class_0 = setmetatable({ -- 87
    __init = function(self, reactor) -- 88
      self.reactor = reactor -- 88
    end, -- 87
    __base = _base_0, -- 87
    __name = "Reactor" -- 87
  }, { -- 87
    __index = _base_0, -- 87
    __call = function(cls, ...) -- 87
      local _self_0 = setmetatable({ }, _base_0) -- 87
      cls.__init(_self_0, ...) -- 87
      return _self_0 -- 87
    end -- 87
  }) -- 87
  _base_0.__class = _class_0 -- 87
  local self = _class_0; -- 87
  self.find = function(self) -- 90
    local reactor = peripheral.find('fissionReactorLogicAdapter') -- 91
    if not (reactor ~= nil) then -- 92
      return nil, 'cannot find fission reactor logic adaptor' -- 93
    end -- 92
    local ret = self.__class(reactor) -- 94
    return ret, nil -- 95
  end -- 90
  Reactor = _class_0 -- 87
end -- 137
return main() -- 139
