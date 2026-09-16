use crate::buffer::Buf;
use crate::data::slot::ItemStack;
use crate::{Bot, Compression};

/// Clientbound Keep Alive (play)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clientbound_Keep_Alive_(play)
pub fn process_keep_alive_packet(buffer: &mut Buf, bot: &mut Bot, compression: &mut Compression) {
    bot.send_packet(write_keep_alive_packet(buffer.read_u64()), compression);
}

/// Disconnect (login/config/play)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect_(login)
pub fn process_kick(buffer: &mut Buf, bot: &mut Bot, _compression: &mut Compression) {
    println!("bot was kicked for \"{}\"", buffer.read_sized_string());
    bot.kicked = true;
}

/// Login (play)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_(play)
pub fn process_join_game(buffer: &mut Buf, bot: &mut Bot, _compression: &mut Compression) {
    bot.entity_id = buffer.read_u32();
}

/// Synchronize Player Position
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Synchronize_Player_Position
pub fn process_teleport(buffer: &mut Buf, bot: &mut Bot, compression: &mut Compression) {
    let id = buffer.read_var_i32();
    let x = buffer.read_f64();
    let y = buffer.read_f64();
    let z = buffer.read_f64();
    let _yaw = buffer.read_f32();
    let _pitch = buffer.read_f32();
    let flags = buffer.read_byte();
    if flags & 0b10000 == 0 {
        bot.x = x;
    } else {
        bot.x += x;
    }
    if flags & 0b01000 == 0 {
        bot.y = y;
    } else {
        bot.y += y;
    }
    if flags & 0b00100 == 0 {
        bot.z = z;
    } else {
        bot.z += z;
    }
    bot.send_packet(write_tele_confirm(id), compression);
    bot.teleported = true;
    println!("{x}, {y}, {z}");
}

/// Chat Message
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Message
pub fn write_chat_message(message: &str) -> Buf {
    // ClientChatMessagePacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x09);

    buf.write_sized_str(message);

    // 1.19 signing fields
    buf.write_u64(0); // timestamp
    buf.write_u64(0); // salt
    buf.write_bool(false); // has signature
    buf.write_var_i32(0); // count
    buf.write_bytes(&[0; 3]); // bitset
    buf.write_var_i32(0); // signature count

    buf
}

/// Swing Arm
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Swing_Arm
pub fn write_animation(off_hand: bool) -> Buf {
    // ClientAnimationPacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x3F);
    buf.write_var_i32(if off_hand { 1 } else { 0 });

    buf
}

/// Player Action (serverbound)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Command
pub fn write_entity_action(entity_id: i32, action_id: i32, jump_boost: i32) -> Buf {
    // ClientEntityActionPacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x2A);

    buf.write_var_i32(entity_id);
    buf.write_var_i32(action_id);
    buf.write_var_i32(jump_boost);

    buf
}

/// Player Input (serverbound)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Input
pub fn write_player_input(flags: u8) -> Buf {
    // ClientPlayerInputPacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x2B);

    buf.write_byte(flags);
    buf
}

/// Set Held Item (serverbound)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Held_Item_(serverbound)
pub fn write_held_slot(slot: u16) -> Buf {
    // ClientHeldItemChangePacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x35);

    buf.write_u16(slot);

    buf
}

/// Confirm Teleportation
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Confirm_Teleportation
pub fn write_tele_confirm(id: i32) -> Buf {
    // ClientTeleportConfirmPacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x00);

    buf.write_var_i32(id);

    buf
}

/// Serverbound Keep Alive (play)
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Serverbound_Keep_Alive_(play)
pub fn write_keep_alive_packet(id: u64) -> Buf {
    // ClientKeepAlivePacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x1C);

    buf.write_u64(id);

    buf
}

pub fn write_current_pos(bot: &Bot) -> Buf {
    write_pos(bot.x, bot.y, bot.z, 0.0, 0.0)
}

/// Set Player Position and Rotation
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Position_and_Rotation
pub fn write_pos(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Buf {
    // ClientPlayerPositionAndRotationPacket
    let mut buf = Buf::new();
    buf.write_packet_id(0x1F);

    buf.write_f64(x);
    buf.write_f64(y);
    buf.write_f64(z);

    buf.write_f32(yaw);
    buf.write_f32(pitch);

    buf.write_bool(false);

    buf
}

/// Set Container Content
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Content
pub fn process_set_container_content(
    buffer: &mut Buf,
    bot: &mut Bot,
    _compression: &mut Compression,
) {
    // id 0 is player inventory
    let window_id = buffer.read_var_i32();
    // need to send back
    let state_id = buffer.read_var_i32();
    let slot_data = buffer.read_slot_array();
    let carry_item = buffer.read_slot();

    bot.state_id = state_id;
    bot.inventory = slot_data.clone();

    println!(
        "debug: window id: {}, state_id:{}, slot data:{:?}, carry item: {:?}",
        window_id, state_id, &slot_data, carry_item
    )
}

/// Click Container
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container
pub fn click_container(
    bot: &mut Bot,
    window_id: i32,
    slot_number: i32,
    slot: ItemStack,
    compression: &mut Compression,
) {
    let mut buf = Buf::new();

    buf.write_packet_id(0x12);
    buf.write_var_i32(window_id);
    buf.write_var_i32(bot.state_id);
    buf.write_u16(slot_number as u16); // Clicked slot index
    buf.write_byte(0); // Button: 0 (Left click)
    buf.write_var_i32(0); // Mode: 0 (Normal pickup / place)

    // Prefix Array (length = 1)
    buf.write_var_i32(1);
    buf.write_u16(slot_number as u16);
    buf.write_bool(false);

    // Carried Item (cursor holds the stack now)
    buf.write_bool(true);
    buf.write_var_i32(slot.item_id);
    buf.write_var_i32(slot.count);
    // Components to add
    buf.write_var_i32(slot.components_to_add.len() as i32);
    for comp in &slot.components_to_add {
        buf.write_var_i32(comp.component_type);
        buf.write_bytes(&comp.raw_data);
    }

    // Components to remove
    buf.write_var_i32(slot.components_to_remove.len() as i32);
    for &type_id in &slot.components_to_remove {
        buf.write_var_i32(type_id);
    }

    bot.send_packet(buf, compression);
}
