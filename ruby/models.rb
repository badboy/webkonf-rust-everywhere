require "ohm"

class TimeTrack < Ohm::Model
  attribute :start
  attribute :stop

  reference :user, :User

  def to_json(*args)
    ({
      id: id,
      start: start,
      stop: stop
    }).to_json(*args)
  end
end

class User < Ohm::Model
  attribute :name
  unique :name

  collection :tracks, :TimeTrack
end

class NullUser
  def name
    nil
  end

  def tracks
    []
  end

  def nil?
    true
  end
end
