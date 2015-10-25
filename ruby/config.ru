#!/usr/bin/env ruby
# encoding: utf-8

require "json"
require "syro"
require "ohm"
require "nobi"

require_relative "models"

FRONTENT_DIR = "../frontend"
SIGNER = Nobi::Signer.new('a3f1194a5a26a1dc1730ad2df7d870f9e11194aba6c08865c38f2a2b65bb8a4b')



class API < Syro::Deck
  def log *args
    $stderr.puts "@@@@ %s" % args.inspect
  end

  def default_headers
    { "Content-Type" => "application/json" }
  end

  def json(object)
    res.write object.to_json
  end

  def unauthorized
    res.session["user-id"] = ""
    res.status = 401
    json(authorized: false, reason: "No such user")
    halt(res.finish)
  end

  def current_user
    @current_user ||= begin
                        user_id = req.session["user-id"]

                        user = fetch_user_or_create(user_id)

                        if user.nil?
                          unauthorized and return
                        end

                        set_user_cookie(user)
                        user
                      end
  end

  def set_user_cookie(user)
    req.session["user-id"] = user.name
  end

  def new_random_user
    retries = 5
    begin
      user_id = rand(100_000_000).to_s
      u = User.create(name: user_id)
      return u
    rescue Ohm::UniqueIndexViolation
      retries -= 1

      if retries == 0
        return nil
      else
        retry
      end
    end
  end

  def fetch_user_or_create(user_id)
    user = User.with(:name, user_id)
    if user_id.nil? || user.nil?
      user = new_random_user
    end

    user
  end
end


app = Syro.new(API) {
  on("time") {
    on("new") {
      user = current_user

      post {
        start = req.params["start"].to_i
        stop = req.params["stop"].to_i
        if start == 0 || stop == 0
          json(error: "Start and stop parameters required.")
        else
          track = TimeTrack.create(start: start,
                                   stop: stop,
                                   user: user)
          json(track)
        end
      }
    }

    on(:track_id) {
      current_user

      get {
        id = inbox[:track_id].to_i
        track = TimeTrack[id]
        json(track)
      }
    }

    get {
      json(current_user.tracks.to_a)
    }
  }
}

if ENV.fetch("RACK_ENV") == "production"
  mount_path = "/legacy"
else
  mount_path = "/"
end

map mount_path do
  secret = ENV.fetch("RACK_SESSION_SECRET", "87998b9378")
  use Rack::MethodOverride
  use Rack::Session::Cookie, secret: secret

  map "/api" do
    run(app)
  end

  run Rack::File.new(FRONTENT_DIR)
end
