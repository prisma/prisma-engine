package queries.orderAndPagination

import org.scalatest.{FlatSpec, Matchers}
import util.ConnectorCapability.JoinRelationLinksCapability
import util._

class PaginationTiebreakerSpec extends FlatSpec with Matchers with ApiSpecBase {

  override def runOnlyForCapabilities: Set[ConnectorCapability] = Set(JoinRelationLinksCapability)

  val project = SchemaDsl.fromStringV11() {
    """model User {
      |  id           String @id @default(cuid())
      |  numFollowers Int
      |  pos          Int
      |}
    """
  }

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    database.setup(project)
    createData()
  }

  //region After

  "After with ties and default order " should "work" in {
    val initial = server.query("""{users{numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3},{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")

    val after = server
      .query(
        """
          |query {
          |  users(  where: {pos: 3}) {
          |    id
          |    numFollowers
          |    pos
          |  }
          |}
        """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
         |{
         |  users(
         |  after: {
         |    id: "$after"
       |    }
         |  ){numFollowers, pos}
         |}
      """,
      project
    )

    result.toString() should be("""{"data":{"users":[{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")
  }

  "After with ties and specific descending order " should "work" in {
    val initial = server.query("""{users(orderBy: numFollowers_DESC){numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6},{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3}]}}""")

    val after = server
      .query(
        """
        |query {
        |  users(  where: {pos: 4}) {
        |    id
        |    numFollowers
        |    pos
        |  }
        |}
      """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
        |{
        |  users(
        |  orderBy: numFollowers_DESC,
        |  after: { id: "$after" }
        |  ){numFollowers, pos}
        |}
      """,
      project
    )

    result.toString() should be(
      """{"data":{"users":[{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6},{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3}]}}""")
  }

  "After with ties and specific ascending order" should "work" in {
    val initial = server.query("""{users(orderBy: numFollowers_ASC){numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3},{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")

    val after = server
      .query(
        """
          |query {
          |  users(  where: {pos: 2}) {
          |    id
          |    numFollowers
          |    pos
          |  }
          |}
        """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
         |{
         |  users(
         |  orderBy: numFollowers_ASC,
         |  after: { id: "$after" }
         |  ){numFollowers, pos}
         |}
      """,
      project
    )

    result.toString() should be(
      """{"data":{"users":[{"numFollowers":9,"pos":3},{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")
  }

  //endregion

  //region Before

  "Before with ties and default order " should "work" in {
    val initial = server.query("""{users{numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3},{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")

    val before = server
      .query(
        """
          |query {
          |  users(  where: {pos: 4}) {
          |    id
          |    numFollowers
          |    pos
          |  }
          |}
        """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
         |{
         |  users(
         |  before: { id: "$before" }
         |  ){numFollowers, pos}
         |}
      """,
      project
    )

    result.toString() should be("""{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3}]}}""")
  }

  "Before with ties and specific descending order " should "work" in {
    val initial = server.query("""{users(orderBy: numFollowers_DESC){numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6},{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3}]}}""")

    val before = server
      .query(
        """
          |query {
          |  users(  where: {pos: 1}) {
          |    id
          |    numFollowers
          |    pos
          |  }
          |}
        """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
         |{
         |  users(
         |  orderBy: numFollowers_DESC,
         |  before: { id: "$before" }
         |  ){numFollowers, pos}
         |}
      """,
      project
    )

    result.toString() should be("""{"data":{"users":[{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")
  }

  "Before with ties and specific ascending order" should "work" in {
    val initial = server.query("""{users(orderBy: numFollowers_ASC){numFollowers, pos}}""", project)

    initial.toString() should be(
      """{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2},{"numFollowers":9,"pos":3},{"numFollowers":10,"pos":4},{"numFollowers":10,"pos":5},{"numFollowers":10,"pos":6}]}}""")

    val before = server
      .query(
        """
          |query {
          |  users(  where: {pos: 3}) {
          |    id
          |    numFollowers
          |    pos
          |  }
          |}
        """,
        project
      )
      .pathAsSeq("data.users")
      .head
      .pathAsString("id")

    val result = server.query(
      s"""
         |{
         |  users(
         |  orderBy: numFollowers_ASC,
         |  before: { id: "$before" }
         |  ){numFollowers, pos}
         |}
      """,
      project
    )

    result.toString() should be("""{"data":{"users":[{"numFollowers":9,"pos":1},{"numFollowers":9,"pos":2}]}}""")
  }

  //endregion

  private def createData(): Unit = {
    server.query("""mutation { createUser(data: {numFollowers: 9, pos: 1}) { id } }""", project)
    server.query("""mutation { createUser(data: {numFollowers: 9, pos: 2}) { id } }""", project)
    server.query("""mutation { createUser(data: {numFollowers: 9, pos: 3}) { id } }""", project)
    server.query("""mutation { createUser(data: {numFollowers: 10, pos: 4}) { id } }""", project)
    server.query("""mutation { createUser(data: {numFollowers: 10, pos: 5}) { id } }""", project)
    server.query("""mutation { createUser(data: {numFollowers: 10, pos: 6}) { id } }""", project)
  }
}
