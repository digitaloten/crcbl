@binding(2) @group(0) var<storage, read_write> control_0 : array<atomic<u32>>;

struct Cmaa2Params_std140_0
{
    @align(16) viewport_x_0 : u32,
    @align(4) viewport_y_0 : u32,
    @align(8) candidate_capacity_0 : u32,
    @align(4) item_capacity_0 : u32,
};

@binding(0) @group(0) var<uniform> params_0 : Cmaa2Params_std140_0;
@binding(4) @group(0) var<storage, read_write> candidates_0 : array<u32>;

@binding(3) @group(0) var<storage, read_write> edges_0 : array<u32>;

@binding(6) @group(0) var<storage, read_write> items_0 : array<u32>;

@binding(1) @group(0) var source_0 : texture_2d<f32>;

@binding(5) @group(0) var<storage, read_write> accum_0 : array<atomic<u32>>;

fn edge_at_0( x_0 : u32,  y_0 : u32) -> u32
{
    var _S1 : bool;
    if(x_0 >= (params_0.viewport_x_0))
    {
        _S1 = true;
    }
    else
    {
        _S1 = y_0 >= (params_0.viewport_y_0);
    }
    if(_S1)
    {
        return u32(0);
    }
    return edges_0[y_0 * params_0.viewport_x_0 + x_0];
}

fn horizontal_turn_0( column_0 : u32,  row_0 : u32) -> i32
{
    var below_0 : bool = (((edge_at_0(column_0, row_0)) & (u32(1)))) != u32(0);
    var above_0 : bool;
    if(row_0 > u32(0))
    {
        above_0 = (((edge_at_0(column_0, row_0 - u32(1))) & (u32(1)))) != u32(0);
    }
    else
    {
        above_0 = false;
    }
    var _S2 : bool;
    if(above_0)
    {
        _S2 = !below_0;
    }
    else
    {
        _S2 = false;
    }
    if(_S2)
    {
        return i32(1);
    }
    if(below_0)
    {
        above_0 = !above_0;
    }
    else
    {
        above_0 = false;
    }
    if(above_0)
    {
        return i32(-1);
    }
    return i32(0);
}

fn boundary_height_0( t_0 : f32,  start_0 : i32,  end_0 : i32,  u_shape_0 : bool) -> f32
{
    if(u_shape_0)
    {
        return -0.5f * f32(start_0) * (2.0f * abs(t_0 - 0.5f)) * 0.5f;
    }
    return -0.5f * f32(start_0) * (1.0f - t_0) - 0.5f * f32(end_0) * t_0;
}

fn append_item_0( target_0 : u32,  from_0 : u32,  share_0 : f32)
{
    if(share_0 < 0.001953125f)
    {
        return;
    }
    var slot_0 : u32 = atomicAdd(&(control_0[i32(1)]), u32(1));
    if(slot_0 < (params_0.item_capacity_0))
    {
        var _S3 : u32 = slot_0 * u32(3);
        items_0[_S3] = target_0;
        items_0[_S3 + u32(1)] = from_0;
        items_0[_S3 + u32(2)] = u32(min(share_0, 0.5f) * 1.048576e+06f);
    }
    return;
}

fn emit_line_0( first_0 : u32,  stride_0 : u32,  len_0 : u32,  offset_0 : u32,  start_1 : i32,  end_1 : i32)
{
    var _S4 : bool;
    if(start_1 == i32(0))
    {
        _S4 = true;
    }
    else
    {
        _S4 = end_1 == i32(0);
    }
    if(_S4)
    {
        return;
    }
    var _S5 : f32 = 1.0f / f32(len_0);
    var _S6 : bool = start_1 == end_1;
    var i_0 : u32 = u32(0);
    for(;;)
    {
        if(i_0 < len_0)
        {
        }
        else
        {
            break;
        }
        var near_0 : f32 = boundary_height_0(f32(i_0) * _S5, start_1, end_1, _S6);
        var _S7 : u32 = i_0 + u32(1);
        var far_0 : f32 = boundary_height_0(f32(_S7) * _S5, start_1, end_1, _S6);
        if(near_0 <= 0.0f)
        {
            _S4 = far_0 <= 0.0f;
        }
        else
        {
            _S4 = false;
        }
        var below_1 : f32;
        var above_1 : f32;
        if(_S4)
        {
            below_1 = -0.5f * (near_0 + far_0);
            above_1 = 0.0f;
        }
        else
        {
            var _S8 : bool;
            if(near_0 >= 0.0f)
            {
                _S8 = far_0 >= 0.0f;
            }
            else
            {
                _S8 = false;
            }
            if(_S8)
            {
                var _S9 : f32 = 0.5f * (near_0 + far_0);
                below_1 = 0.0f;
                above_1 = _S9;
            }
            else
            {
                var crossing_0 : f32 = near_0 / (near_0 - far_0);
                var _S10 : f32 = 0.5f * abs(min(near_0, far_0));
                if(near_0 < 0.0f)
                {
                    below_1 = crossing_0;
                }
                else
                {
                    below_1 = 1.0f - crossing_0;
                }
                var _S11 : f32 = _S10 * below_1;
                var _S12 : f32 = 0.5f * max(near_0, far_0);
                if(near_0 > 0.0f)
                {
                    above_1 = crossing_0;
                }
                else
                {
                    above_1 = 1.0f - crossing_0;
                }
                var _S13 : f32 = _S12 * above_1;
                below_1 = _S11;
                above_1 = _S13;
            }
        }
        var pixel_0 : u32 = first_0 + i_0 * stride_0;
        var _S14 : u32 = pixel_0 - offset_0;
        append_item_0(pixel_0, _S14, below_1);
        append_item_0(_S14, pixel_0, above_1);
        i_0 = _S7;
    }
    return;
}

fn vertical_turn_0( column_1 : u32,  row_1 : u32) -> i32
{
    var right_0 : bool = (((edge_at_0(column_1, row_1)) & (u32(2)))) != u32(0);
    var left_0 : bool;
    if(column_1 > u32(0))
    {
        left_0 = (((edge_at_0(column_1 - u32(1), row_1)) & (u32(2)))) != u32(0);
    }
    else
    {
        left_0 = false;
    }
    var _S15 : bool;
    if(left_0)
    {
        _S15 = !right_0;
    }
    else
    {
        _S15 = false;
    }
    if(_S15)
    {
        return i32(1);
    }
    if(right_0)
    {
        left_0 = !left_0;
    }
    else
    {
        left_0 = false;
    }
    if(left_0)
    {
        return i32(-1);
    }
    return i32(0);
}

@compute
@workgroup_size(64, 1, 1)
fn shapesMain(@builtin(global_invocation_id) thread_0 : vec3<u32>)
{
    var _S16 : bool;
    var _S17 : bool;
    var _S18 : u32 = atomicLoad(&(control_0[i32(0)]));
    var slot_1 : u32 = thread_0.x;
    if(slot_1 >= (min(_S18, params_0.candidate_capacity_0)))
    {
        return;
    }
    var index_0 : u32 = candidates_0[slot_1];
    var width_0 : u32 = params_0.viewport_x_0;
    var x_1 : u32 = candidates_0[slot_1] % params_0.viewport_x_0;
    var y_1 : u32 = index_0 / width_0;
    var own_0 : u32 = edges_0[index_0];
    var _S19 : bool;
    if(((edges_0[index_0] & (u32(2)))) != u32(0))
    {
        _S19 = (((edge_at_0(x_1 - u32(1), y_1)) & (u32(2)))) == u32(0);
    }
    else
    {
        _S19 = false;
    }
    var len_1 : u32;
    if(_S19)
    {
        len_1 = u32(1);
        for(;;)
        {
            var _S20 : bool = len_1 <= u32(64);
            _S16 = _S20;
            if(_S20)
            {
                _S19 = (((edge_at_0(x_1 + len_1, y_1)) & (u32(2)))) != u32(0);
            }
            else
            {
                _S19 = false;
            }
            if(_S19)
            {
            }
            else
            {
                break;
            }
            len_1 = len_1 + u32(1);
        }
        if(_S16)
        {
            emit_line_0(index_0, u32(1), len_1, width_0, horizontal_turn_0(x_1, y_1), horizontal_turn_0(x_1 + len_1, y_1));
        }
    }
    if(((own_0 & (u32(1)))) != u32(0))
    {
        _S19 = (((edge_at_0(x_1, y_1 - u32(1))) & (u32(1)))) == u32(0);
    }
    else
    {
        _S19 = false;
    }
    if(_S19)
    {
        len_1 = u32(1);
        for(;;)
        {
            var _S21 : bool = len_1 <= u32(64);
            _S17 = _S21;
            if(_S21)
            {
                _S19 = (((edge_at_0(x_1, y_1 + len_1)) & (u32(1)))) != u32(0);
            }
            else
            {
                _S19 = false;
            }
            if(_S19)
            {
            }
            else
            {
                break;
            }
            len_1 = len_1 + u32(1);
        }
        if(_S17)
        {
            emit_line_0(index_0, width_0, len_1, u32(1), vertical_turn_0(x_1, y_1), vertical_turn_0(x_1, y_1 + len_1));
        }
    }
    return;
}

@compute
@workgroup_size(64, 1, 1)
fn accumulateMain(@builtin(global_invocation_id) thread_1 : vec3<u32>)
{
    var _S22 : u32 = atomicLoad(&(control_0[i32(1)]));
    var slot_2 : u32 = thread_1.x;
    if(slot_2 >= (min(_S22, params_0.item_capacity_0)))
    {
        return;
    }
    var _S23 : u32 = slot_2 * u32(3);
    var target_1 : u32 = items_0[_S23];
    var from_1 : u32 = items_0[_S23 + u32(1)];
    var weight_0 : u32 = items_0[_S23 + u32(2)];
    var pixels_0 : u32 = params_0.viewport_x_0 * params_0.viewport_y_0;
    var _S24 : bool;
    if(items_0[_S23] >= pixels_0)
    {
        _S24 = true;
    }
    else
    {
        _S24 = from_1 >= pixels_0;
    }
    if(_S24)
    {
        return;
    }
    var width_1 : u32 = params_0.viewport_x_0;
    var _S25 : u32 = from_1 % params_0.viewport_x_0;
    var _S26 : i32 = i32(_S25);
    var _S27 : u32 = from_1 / width_1;
    var _S28 : vec3<i32> = vec3<i32>(vec2<i32>(_S26, i32(_S27)), i32(0));
    var color_0 : vec3<f32> = saturate((textureLoad((source_0), ((_S28)).xy, ((_S28)).z)).xyz);
    var share_1 : f32 = f32(weight_0);
    var _S29 : u32 = target_1 * u32(4);
    var _S30 : u32 = atomicAdd(&(accum_0[_S29]), u32(color_0.x * share_1));
    var _S31 : u32 = atomicAdd(&(accum_0[_S29 + u32(1)]), u32(color_0.y * share_1));
    var _S32 : u32 = atomicAdd(&(accum_0[_S29 + u32(2)]), u32(color_0.z * share_1));
    var _S33 : u32 = atomicAdd(&(accum_0[_S29 + u32(3)]), weight_0);
    return;
}

